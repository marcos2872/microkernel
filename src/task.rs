use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;
use core::arch::global_asm;

global_asm!(include_str!("task/context.s"));

extern "C" {
    pub fn context_switch(current: *mut TaskContext, next: *const TaskContext);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Ready,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaskContext {
    pub rsp: VirtAddr,
    pub rbp: VirtAddr,
    pub rbx: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: VirtAddr,
}

#[derive(Clone, Copy)]
struct UnsafeSendSync<T>(T);

unsafe impl<T> Send for UnsafeSendSync<T> {}
unsafe impl<T> Sync for UnsafeSendSync<T> {}

pub struct Task {
    id: TaskId,
    pub state: TaskState,
    pub context: TaskContext,
    p4_table: UnsafeSendSync<*mut PageTable>,
}

impl Task {
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

pub struct Scheduler {
    tasks: alloc::vec::Vec<Task>,
    current_task: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: alloc::vec::Vec::new(),
            current_task: 0,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

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
