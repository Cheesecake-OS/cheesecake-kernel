#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use core::hint::spin_loop;
use core::panic::PanicInfo;

mod cpu;
mod input;
mod kernel;
mod mm;
mod serial;
mod tasks;
mod vga;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn keyboard_task_entry() -> ! {
    loop {
        if let Some(ch) = input::keyboard::poll() {
            vga::print_char(ch);
        }
        crate::tasks::scheduler::switch();
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial::init();
    serial_println!("[ OK ] Serial Initialized");
    let phys_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("physical_memory_offset not mapped — check BOOTLOADER_CONFIG")
        as usize;
    let fb = boot_info.framebuffer.as_mut().unwrap();
    let fb_ptr = fb as *mut _;
    vga::init(unsafe { &mut *fb_ptr });
    serial_println!("[ OK ] VGA Initialized");
    vga::clear();
    serial_println!("[ OK ] VGA cleared");

    let k = kernel::Kernel::init(boot_info);
    k.print_banner();
    serial_println!("[ OK ] Kernel Initialized");
    input::keyboard::init();
    serial_println!("[ OK ] Keyboard Initialized");

    {
        let mut sched = crate::tasks::scheduler::SCHEDULER.lock();
        sched.spawn(keyboard_task_entry, 0, phys_offset);
    }

    serial_println!("Cheesecake Kernel ready.");
    vga::print("Cheesecake Kernel ready.\n");

    loop {
        // Yield execution back and forth between the main loop and your task
        crate::tasks::scheduler::switch();

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
