use crate::mm::buddy::BUDDY;
use crate::serial_println;
use crate::tasks::scheduler::exit_current;

pub const DEFAULT_STACK_ORDER: usize = 2; // 16 KiB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    // Probably add some other useful states, idk
    Ready,
    Running,
    Blocked,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbx: usize,
    pub rbp: usize,
    pub rip: usize, // entry point, jumped to via ret
}

pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub stack_bottom: usize, // physical address of allocation base
    pub stack_ptr: usize,    // virtual address of saved RSP
    pub stack_order: usize,  // buddy allocator order used for this stack
}

impl Task {
    pub fn new(
        id: TaskId,
        entry_point: fn(), // fn() tasks can return
        stack_order: usize,
        phys_mem_offset: usize,
    ) -> Self {
        let stack_bottom_phys = BUDDY
            .alloc(stack_order)
            .expect("failed to allocate task stack");

        let stack_size = 4096 << stack_order;
        let stack_bottom_virt = stack_bottom_phys + phys_mem_offset;
        let stack_top_virt = stack_bottom_virt + stack_size;

        let stack_top_aligned = stack_top_virt & !0xF;

        let exit_slot = stack_top_aligned - core::mem::size_of::<usize>();
        unsafe {
            (exit_slot as *mut usize).write(exit_current as usize);
        }

        let context_size = core::mem::size_of::<TaskContext>();
        let stack_ptr_virt = exit_slot - context_size;

        let context = TaskContext {
            rip: entry_point as usize,
            ..Default::default()
        };
        unsafe {
            (stack_ptr_virt as *mut TaskContext).write(context);
        }

        Task {
            id,
            state: TaskState::Ready,
            stack_bottom: stack_bottom_phys,
            stack_ptr: stack_ptr_virt,
            stack_order,
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        // Don't free kernel dummy task.
        if self.stack_bottom != 0 {
            BUDDY.free(self.stack_bottom, self.stack_order);
            serial_println!("[SCHED] Task {} stack freed.", self.id.0);
        }
    }
}
