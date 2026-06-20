#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::hint::spin_loop;
use core::panic::PanicInfo;

mod cpu;
mod input;
mod kernel;
mod mm;
mod serial;
mod vga;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    serial::init();
    vga::clear();

    let k = kernel::Kernel::init(boot_info);
    k.print_banner();

    serial_println!("Cheesecake Kernel ready.");
    vga::print("Cheesecake Kernel ready.\n");

    loop {
        if let Some(ch) = input::keyboard::poll() {
            vga::print_char(ch);
        }
        spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
