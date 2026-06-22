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
mod syscalls;
mod tasks;
mod vga;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn print_key_task() {
    loop {
        if let Some(ch) = input::keyboard::poll() {
            vga::print_char(ch);
            serial_println!("[KEY] '{}'", ch);
            break;
        }
        crate::tasks::scheduler::switch();
    }
    // Returns here → exit_current() via trampoline
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Init Start
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
    vga::print_ok("VGA Initialized");

    let k = kernel::Kernel::init(boot_info, phys_offset);
    serial_println!("[ OK ] Kernel Initialized");
    vga::print_ok("Kernel Initialized");

    input::keyboard::init();
    serial_println!("[ OK ] Keyboard Initialized");
    vga::print_ok("Keyboard Initialized");

    vga::clear();
    // Init End

    // Misc stuff
    k.print_banner();
    serial_println!("Cheesecake Kernel ready.");
    vga::print("Cheesecake Kernel ready.\n");

    // #### Code starts ####

    {
        let mut sched = tasks::scheduler::SCHEDULER.lock();
        sched.init(phys_offset);
        sched.spawn(main_task); // spawn the real entry point as a task
    }

    tasks::scheduler::run_first();
}
fn main_task() {
    serial_println!("Main task started");
    loop {
        if tasks::scheduler::SCHEDULER.lock().ready_count() == 0 {
            tasks::scheduler::SCHEDULER.lock().spawn(print_key_task);
        }
        crate::tasks::scheduler::switch();
        spin_loop(); // yield CPU
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
