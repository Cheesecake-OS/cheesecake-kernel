use crate::serial_println;
use linked_list_allocator::LockedHeap;

pub const HEAP_SIZE: usize = 512 * 1024; // 512 KiB static heap

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Lives in .bss, always mapped by the bootloader
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

pub fn init() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_MEM.as_mut_ptr(), HEAP_SIZE);
    }
    serial_println!(
        "Heap at 0x{:x}, size {} KiB",
        unsafe { HEAP_MEM.as_ptr() as usize },
        HEAP_SIZE / 1024
    );
}
