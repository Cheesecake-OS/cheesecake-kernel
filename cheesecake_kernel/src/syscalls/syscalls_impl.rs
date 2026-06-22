use crate::tasks;
use crate::vga;

pub fn sys_read(args: [u64; 6]) -> u64 {
    unimplemented!("Syscall not implemented: read");
}

pub fn sys_write(args: [u64; 6]) -> u64 {
    let fd = args[0];
    let buf = args[1] as *const u8;
    let len = args[2];

    match fd {
        1 => {
            for i in 0..len {
                let c = unsafe { *buf.add(i as usize) };
                vga::print_char(c as char);
            }
            len
        }
        _ => 0,
    }
}

pub fn sys_open(args: [u64; 6]) -> u64 {
    unimplemented!("Syscall not implemented: open");
}

pub fn sys_close(args: [u64; 6]) -> u64 {
    unimplemented!("Syscall not implemented: close");
}

pub fn sys_exit(args: [u64; 6]) -> u64 {
    unimplemented!("Syscall not implemented: exit")
    // let code = args[0];

    // tasks::scheduler::SCHEDULER.lock().exit_current(code);

    // 0
}

pub fn sys_yield(_args: [u64; 6]) -> u64 {
    crate::tasks::scheduler::switch();
    0
}

pub fn sys_getpid(_args: [u64; 6]) -> u64 {
    tasks::scheduler::SCHEDULER
        .lock()
        .get_current_task_pid()
        .map(|pid| pid.0 as u64)
        .unwrap_or(0)
}
pub fn sys_mmap(_args: [u64; 6]) -> u64 {
    unimplemented!("Syscall not implemented: mmap")
}
