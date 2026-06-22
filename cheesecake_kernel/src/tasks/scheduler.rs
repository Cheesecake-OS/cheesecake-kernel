extern crate alloc;

use crate::tasks::context_switch;
use crate::tasks::task::{Task, TaskId, TaskState, DEFAULT_STACK_ORDER};
use alloc::collections::VecDeque;
use spin::Mutex;

static PHYS_MEM_OFFSET: Mutex<usize> = Mutex::new(0);

pub struct Scheduler {
    // TODO: Add multi-core support, somehow, and also multi-threading
    ready_queue: VecDeque<Task>,
    current_task: Option<Task>,
    next_id: usize,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            ready_queue: VecDeque::new(),
            current_task: None,
            next_id: 1,
        }
    }

    pub fn init(&mut self, phys_mem_offset: usize) {
        *PHYS_MEM_OFFSET.lock() = phys_mem_offset;

        // Kernel main thread, no real stack
        self.current_task = Some(Task {
            id: TaskId(0),
            state: TaskState::Running,
            stack_bottom: 0,
            stack_ptr: 0,
            stack_order: 0,
        });
    }

    pub fn spawn(&mut self, entry: fn()) {
        self.spawn_with_order(entry, DEFAULT_STACK_ORDER);
    }

    pub fn spawn_with_order(&mut self, entry: fn(), order: usize) {
        let id = TaskId(self.next_id);
        self.next_id += 1;
        let offset = *PHYS_MEM_OFFSET.lock();
        let task = Task::new(id, entry, order, offset);
        self.ready_queue.push_back(task);
    }

    pub fn current_id(&self) -> Option<TaskId> {
        self.current_task.as_ref().map(|t| t.id)
    }

    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }
    pub fn get_current_task_pid(&self) -> Option<TaskId> {
        self.current_task.as_ref().map(|t| t.id)
    }
}

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

pub fn switch() {
    let old_rsp_ptr: *mut usize;
    let new_rsp: usize;

    {
        let mut sched = SCHEDULER.lock();

        if sched.ready_queue.is_empty() {
            return;
        }

        let mut old_task = sched.current_task.take().unwrap();
        old_task.state = TaskState::Ready;
        let is_real = old_task.stack_bottom != 0;

        sched.ready_queue.push_back(old_task);

        old_rsp_ptr = if is_real {
            &mut sched.ready_queue.back_mut().unwrap().stack_ptr as *mut usize
        } else {
            core::ptr::null_mut()
        };

        let mut next_task = sched.ready_queue.pop_front().unwrap();
        next_task.state = TaskState::Running;
        new_rsp = next_task.stack_ptr;
        sched.current_task = Some(next_task);
    }

    unsafe {
        context_switch(old_rsp_ptr, new_rsp);
    }
}

pub fn exit_current() -> ! {
    let new_rsp: usize;

    {
        let mut sched = SCHEDULER.lock();

        sched.current_task.take();

        if sched.ready_queue.is_empty() {
            // Idle
            drop(sched);
            loop {
                unsafe {
                    core::arch::asm!("hlt");
                }
            }
        }

        let mut next = sched.ready_queue.pop_front().unwrap();
        next.state = TaskState::Running;
        new_rsp = next.stack_ptr;
        sched.current_task = Some(next);
    }

    unsafe {
        core::arch::asm!(
            "mov rsp, {rsp}",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp",
            "ret",
            rsp = in(reg) new_rsp,
            options(noreturn)
        );
    }
}

pub fn run_first() -> ! {
    let new_rsp: usize;
    {
        let mut sched = SCHEDULER.lock();
        let mut next = sched.ready_queue.pop_front().unwrap();
        next.state = TaskState::Running;
        new_rsp = next.stack_ptr;
        sched.current_task = Some(next);
    }
    unsafe {
        core::arch::asm!(
            "mov rsp, {rsp}",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp",
            "ret",
            rsp = in(reg) new_rsp,
            options(noreturn)
        );
    }
}
