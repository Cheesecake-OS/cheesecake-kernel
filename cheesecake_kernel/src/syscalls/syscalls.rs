// Important: Never change the order of these syscalls, and also, don't add logic to this file

// File system
pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_OPEN: u64 = 2; // Unimplemented for now
pub const SYS_CLOSE: u64 = 3; // Unimplemented for now

// Process management
pub const SYS_EXIT: u64 = 4;
pub const SYS_YIELD: u64 = 5;
pub const SYS_GETPID: u64 = 6; // Wait for userspace
pub const SYS_MMAP: u64 = 7; // Unimplemented for now
