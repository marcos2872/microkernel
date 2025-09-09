//! Este módulo implementa o scheduler e o gerenciamento de tarefas.

use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;
use core::arch::global_asm;

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
    id: TaskId,
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
    tasks: alloc::vec::Vec<Task>,
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

    /// Adiciona uma nova tarefa ao scheduler.
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Seleciona a próxima tarefa a ser executada.
    ///
    /// Retorna uma tupla com os contextos da tarefa atual e da próxima tarefa.
    pub fn schedule(&mut self) -> Option<(&mut TaskContext, &TaskContext)> {
        let current_task_index = self.current_task;
        if self.tasks.len() <= 1 {
            return None;
        }
        let next_task_index = (current_task_index + 1) % self.tasks.len();
        self.current_task = next_task_index;

        let (current_slice, next_slice) = self.tasks.split_at_mut(current_task_index);
        let current_task = &mut current_slice[0];
        let next_task = &mut next_slice[next_task_index - current_task_index];

        current_task.state = TaskState::Ready;
        next_task.state = TaskState::Running;

        Some((&mut current_task.context, &next_task.context))
    }
}
