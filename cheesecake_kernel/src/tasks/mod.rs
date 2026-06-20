pub mod scheduler;
pub mod task;

use core::arch::global_asm;

global_asm!(include_str!("switch.s"));

extern "C" {
    pub fn context_switch(old_rsp: *mut usize, new_rsp: usize);
}
