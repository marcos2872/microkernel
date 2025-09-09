//! Este módulo implementa o scheduler e o gerenciamento de tarefas.

use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;
use core::arch::global_asm;
use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::collections::BTreeMap;

// Inclui o código assembly para a troca de contexto.
global_asm!(include_str!("task/context.s"));

extern "C" {
    /// Realiza a troca de contexto entre duas tarefas.
    ///
    /// Salva o estado da tarefa atual e restaura o estado da próxima tarefa.
    /// Esta função é implementada em assembly (`context.s`).
    ///
    /// # Safety
    ///
    /// Esta função é extremamente insegura e deve ser chamada com muito cuidado.
    /// Os ponteiros de contexto devem ser válidos.
    pub fn context_switch(current: *mut TaskContext, next: *const TaskContext);
}

/// Um identificador único para uma tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    /// Cria um novo `TaskId` único.
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// O estado de uma tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// A tarefa está atualmente em execução.
    Running,
    /// A tarefa está pronta para ser executada.
    Ready,
    /// A tarefa está bloqueada, esperando por um evento.
    Blocked,
}

/// Representa uma mensagem que pode ser enviada entre tarefas.
pub type Message = u64;

lazy_static! {
    /// O gerenciador de caixas de correio (mailboxes) global.
    ///
    /// Mapeia cada `TaskId` a uma fila de mensagens (`VecDeque`).
    static ref MAILBOXES: Mutex<BTreeMap<TaskId, VecDeque<Message>>> =
        Mutex::new(BTreeMap::new());
}

/// O contexto de uma tarefa, contendo o estado dos registradores da CPU.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    /// Ponteiro da pilha (RSP).
    pub rsp: VirtAddr,
    /// Ponteiro base da pilha (RBP).
    pub rbp: VirtAddr,
    /// Registrador RBX.
    pub rbx: u64,
    /// Registrador R12.
    pub r12: u64,
    /// Registrador R13.
    pub r13: u64,
    /// Registrador R14.
    pub r14: u64,
    /// Registrador R15.
    pub r15: u64,
    /// Ponteiro de instrução (RIP).
    pub rip: VirtAddr,
}

/// Wrapper para permitir que um tipo seja `Send` e `Sync`.
///
/// # Safety
///
/// O chamador deve garantir que o uso deste wrapper não viole as regras de
/// segurança de threads.
#[derive(Clone, Copy)]
struct UnsafeSendSync<T>(T);

unsafe impl<T> Send for UnsafeSendSync<T> {}
unsafe impl<T> Sync for UnsafeSendSync<T> {}

/// Representa uma tarefa no sistema.
pub struct Task {
    /// O ID único da tarefa.
    pub id: TaskId,
    /// O estado atual da tarefa.
    pub state: TaskState,
    /// O contexto da CPU da tarefa.
    pub context: TaskContext,
    /// Ponteiro para a tabela de páginas P4 da tarefa.
    p4_table: UnsafeSendSync<*mut PageTable>,
}

impl Task {
    /// Cria uma nova `Task`.
    pub fn new(entry_point: VirtAddr, stack_top: VirtAddr, p4_table: *mut PageTable) -> Self {
        Task {
            id: TaskId::new(),
            state: TaskState::Ready,
            context: TaskContext {
                rsp: stack_top,
                rbp: stack_top,
                rbx: 0,
                r12: 0,
                r13: 0,
                r14: 0,
                r15: 0,
                rip: entry_point,
            },
            p4_table: UnsafeSendSync(p4_table),
        }
    }
}

/// O scheduler de tarefas.
///
/// Implementa uma política de escalonamento round-robin simples.
pub struct Scheduler {
    pub tasks: alloc::vec::Vec<Task>,
    current_task: usize,
}

impl Scheduler {
    /// Cria um novo `Scheduler`.
    pub fn new() -> Self {
        Scheduler {
            tasks: alloc::vec::Vec::new(),
            current_task: 0,
        }
    }

    /// Adiciona uma nova tarefa ao scheduler e cria uma mailbox para ela.
    pub fn add_task(&mut self, task: Task) {
        let task_id = task.id;
        self.tasks.push(task);
        MAILBOXES.lock().insert(task_id, VecDeque::new());
    }

    /// Seleciona a próxima tarefa a ser executada.
    ///
    /// Retorna uma tupla com os contextos da tarefa atual e da próxima tarefa.
    pub fn schedule(&mut self) -> Option<(&mut TaskContext, &TaskContext)> {
        let current_task_index = self.current_task;
        let mut next_task_index = (current_task_index + 1) % self.tasks.len();

        while self.tasks[next_task_index].state != TaskState::Ready {
            next_task_index = (next_task_index + 1) % self.tasks.len();
            if next_task_index == current_task_index {
                // No other task is ready
                return None;
            }
        }

        self.current_task = next_task_index;

        let (current_task, next_task) = {
            let tasks = &mut self.tasks;
            let (first, second) = tasks.split_at_mut(core::cmp::max(current_task_index, next_task_index));
            if current_task_index < next_task_index {
                (&mut first[current_task_index], &mut second[0])
            } else {
                (&mut second[0], &mut first[next_task_index])
            }
        };

        if current_task.state == TaskState::Running {
            current_task.state = TaskState::Ready;
        }
        next_task.state = TaskState::Running;

        Some((&mut current_task.context, &next_task.context))
    }
}

use crate::interrupts::InterruptIndex;

/// Envia uma mensagem para uma tarefa.
///
/// Se a tarefa receptora estiver bloqueada, ela é acordada.
/// Retorna `true` se a mensagem foi enviada com sucesso.
pub fn send(receiver_id: TaskId, message: Message) -> bool {
    let mut mailboxes = MAILBOXES.lock();
    if let Some(mailbox) = mailboxes.get_mut(&receiver_id) {
        mailbox.push_back(message);
        // Wake up the receiver if it was blocked
        let mut scheduler = crate::SCHEDULER.lock();
        if let Some(task) = scheduler.tasks.iter_mut().find(|t| t.id == receiver_id) {
            if task.state == TaskState::Blocked {
                task.state = TaskState::Ready;
            }
        }
        true
    } else {
        false
    }
}

/// Recebe uma mensagem da mailbox da tarefa atual.
///
/// Se a mailbox estiver vazia, a tarefa é bloqueada até que uma mensagem chegue.
pub fn receive() -> Message {
    let my_id = crate::SCHEDULER.lock().current_task_id();
    loop {
        let mut mailboxes = MAILBOXES.lock();
        if let Some(msg) = mailboxes.get_mut(&my_id).unwrap().pop_front() {
            return msg;
        }
        drop(mailboxes);

        // Block the task and yield
        {
            let mut scheduler = crate::SCHEDULER.lock();
            let current_task = scheduler.tasks.iter_mut().find(|t| t.id == my_id).unwrap();
            current_task.state = TaskState::Blocked;
        }
        unsafe {
            core::arch::asm!("int {}", const InterruptIndex::Timer as u8);
        }
    }
}

impl Scheduler {
    /// Retorna o ID da tarefa atual.
    pub fn current_task_id(&self) -> TaskId {
        self.tasks[self.current_task].id
    }
}

/// Cede o tempo de CPU da tarefa atual para o scheduler.
pub fn yield_now() {
    unsafe {
        core::arch::asm!("int {}", const InterruptIndex::Timer as u8);
    }
}
