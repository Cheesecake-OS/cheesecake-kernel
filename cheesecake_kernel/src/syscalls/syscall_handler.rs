use super::syscalls::*;
use super::syscalls_impl::*;
use crate::serial_println;

pub fn syscall_handler(syscall_id: u64, args: [u64; 6]) -> u64 {
    match syscall_id {
        SYS_READ => sys_read(args),
        SYS_WRITE => sys_write(args),
        SYS_OPEN => sys_open(args),
        SYS_CLOSE => sys_close(args),
        SYS_EXIT => sys_exit(args),
        SYS_YIELD => sys_yield(args),
        SYS_GETPID => sys_getpid(args),
        SYS_MMAP => sys_mmap(args),
        _ => {
            serial_println!("[ SYS ] Unknown syscall: {}", syscall_id);
            0
        }
    }
}
