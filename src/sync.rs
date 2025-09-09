//! Módulo de primitivas de sincronização.

use crate::task::{self, TaskId, TaskState};
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

/// Um semáforo para sincronização entre tarefas.
pub struct Semaphore {
    counter: AtomicUsize,
    waiting_tasks: Mutex<VecDeque<TaskId>>,
}

impl Semaphore {
    /// Cria um novo semáforo com um valor inicial.
    pub const fn new(initial_value: usize) -> Self {
        Semaphore {
            counter: AtomicUsize::new(initial_value),
            waiting_tasks: Mutex::new(VecDeque::new()),
        }
    }

    /// Operação "down" (ou "wait").
    ///
    /// Decrementa o contador do semáforo. Se o contador for zero, a tarefa
    /// atual é bloqueada até que outra tarefa chame `up`.
    pub fn down(&self) {
        loop {
            let mut value = self.counter.load(Ordering::Relaxed);
            if value > 0 {
                if self.counter.compare_exchange(value, value - 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                    return;
                }
                continue;
            }

            let my_id = crate::SCHEDULER.lock().current_task_id();
            self.waiting_tasks.lock().push_back(my_id);

            // Block the task
            {
                let mut scheduler = crate::SCHEDULER.lock();
                let current_task = scheduler.tasks.iter_mut().find(|t| t.id == my_id).unwrap();
                current_task.state = TaskState::Blocked;
            }
            // Yield to another task
            task::yield_now();
        }
    }

    /// Operação "up" (ou "signal").
    ///
    /// Incrementa o contador do semáforo e acorda uma tarefa que esteja
    /// esperando, se houver alguma.
    pub fn up(&self) {
        self.counter.fetch_add(1, Ordering::Release);
        if let Some(task_id) = self.waiting_tasks.lock().pop_front() {
            let mut scheduler = crate::SCHEDULER.lock();
            if let Some(task) = scheduler.tasks.iter_mut().find(|t| t.id == task_id) {
                task.state = TaskState::Ready;
            }
        }
    }
}
