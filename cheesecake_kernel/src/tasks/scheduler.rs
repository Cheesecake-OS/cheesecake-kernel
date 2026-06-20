extern crate alloc;
use crate::tasks::context_switch;
use crate::tasks::task::{Task, TaskId, TaskState};
use alloc::collections::VecDeque;
use spin::Mutex;

pub struct Scheduler {
    // TODO: Add mutli-core support, somehow, and also multi-threading
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

    pub fn init(&mut self) {
        let kernel_dummy = Task {
            id: TaskId(0),
            state: TaskState::Running,
            stack_bottom: 0,
            stack_ptr: 0,
            stack_order: 0,
        };
        self.current_task = Some(kernel_dummy);
    }

    pub fn spawn(&mut self, entry: fn() -> !, order: usize, phys_mem_offset: usize) {
        let id = TaskId(self.next_id);
        self.next_id += 1;
        let task = Task::new(id, entry, order, phys_mem_offset);
        self.ready_queue.push_back(task);
    }
}

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

pub fn switch() {
    let mut old_rsp_ptr: *mut usize = core::ptr::null_mut();
    let mut new_rsp: usize = 0;

    {
        let mut sched = SCHEDULER.lock();
        if sched.ready_queue.is_empty() {
            return;
        }

        let mut old_task = sched.current_task.take().unwrap();
        old_task.state = TaskState::Ready;
        sched.ready_queue.push_back(old_task);

        let mut next_task = sched.ready_queue.pop_front().unwrap();
        next_task.state = TaskState::Running;

        new_rsp = next_task.stack_ptr;
        old_rsp_ptr = &mut sched.ready_queue.back_mut().unwrap().stack_ptr as *mut usize;

        sched.current_task = Some(next_task);
    }

    unsafe {
        context_switch(old_rsp_ptr, new_rsp);
    }
}
