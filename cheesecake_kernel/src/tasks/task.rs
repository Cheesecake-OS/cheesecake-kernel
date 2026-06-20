use crate::mm::buddy::BUDDY;
use crate::serial_println;
use core::num::NonZeroUsize;

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
#[repr(C, packed)]
pub struct TaskContext {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbx: usize,
    rbp: usize,
    rip: usize, // Instruction pointer (where to jump next)
}

pub struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub stack_bottom: usize, // Lowest address of the allocated memory block
    pub stack_ptr: usize,    // Current top of stack (saved $RSP)
    pub stack_order: usize,  // The buddy allocator order used for this stack
}

impl Task {
    pub fn new(
        id: TaskId,
        entry_point: fn() -> !,
        stack_order: usize,
        phys_mem_offset: usize,
    ) -> Self {
        let stack_bottom_phys = BUDDY
            .alloc(stack_order)
            .expect("Failed to allocate stack for new task");

        let stack_size = 4096 << stack_order;

        // Physical addresses to virtual addresses
        let stack_bottom_virt = stack_bottom_phys + phys_mem_offset;
        let stack_top_virt = stack_bottom_virt + stack_size;

        let mut context = TaskContext::default();
        context.rip = entry_point as usize;

        let context_size = core::mem::size_of::<TaskContext>();
        let stack_ptr_virt = stack_top_virt - context_size;

        unsafe {
            let context_ptr = stack_ptr_virt as *mut TaskContext;
            context_ptr.write(context);
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
        BUDDY.free(self.stack_bottom, self.stack_order);
        serial_println!("Task {} memory reclaimed cleanly.", self.id.0);
    }
}
