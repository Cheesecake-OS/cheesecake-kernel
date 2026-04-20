#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use cheesecake_kernel::vga::print;
use core::panic::PanicInfo;
use core::arch::x86_64::__cpuid;

mod vga;
mod serial;

entry_point!(kernel_main);

static mut CPU_VENDOR: [u8; 12] = [0; 12];

pub fn get_cpu_vendor() -> &'static str {
    // Example: AuthenticAMD in my case
    use core::arch::x86_64::__cpuid;

    let res = unsafe { __cpuid(0) };

    unsafe {
        CPU_VENDOR[0..4].copy_from_slice(&res.ebx.to_le_bytes());
        CPU_VENDOR[4..8].copy_from_slice(&res.edx.to_le_bytes());
        CPU_VENDOR[8..12].copy_from_slice(&res.ecx.to_le_bytes());

        core::str::from_utf8_unchecked(&CPU_VENDOR)
    }
}
pub fn get_cpu_brand() -> [u8; 48] {
    // Full CPU model, like AMD Ryzen 7 3750H with Radeon Vega Mobile Gfx
    let mut brand = [0u8; 48];

    unsafe {
        let r0 = __cpuid(0x80000002);
        let r1 = __cpuid(0x80000003);
        let r2 = __cpuid(0x80000004);

        let mut write = |r: core::arch::x86_64::CpuidResult, offset: usize| {
            brand[offset..offset+4].copy_from_slice(&r.eax.to_le_bytes());
            brand[offset+4..offset+8].copy_from_slice(&r.ebx.to_le_bytes());
            brand[offset+8..offset+12].copy_from_slice(&r.ecx.to_le_bytes());
            brand[offset+12..offset+16].copy_from_slice(&r.edx.to_le_bytes());
        };

        write(r0, 0);
        write(r1, 16);
        write(r2, 32);
    }

    brand
}

fn print_system_info(boot_info: &'static BootInfo) {
    

    serial_println!("=== System Info ===");

    // CPU
    let cpuid = get_cpu_vendor();
    serial_println!("CPU Vendor: {}", cpuid);
    let brand = get_cpu_brand();
    serial_println!("CPU Brand: {}", core::str::from_utf8(&brand).unwrap());



    // RAM
    let mut total = 0;
    for region in boot_info.memory_map.iter() {
        if region.region_type == bootloader::bootinfo::MemoryRegionType::Usable {
            total += region.range.end_addr() - region.range.start_addr();
        }
    }

    serial_println!("Usable RAM: {} MB", total / 1024 / 1024);
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    serial::init();

    serial_println!("Kernel booted!");
    vga::clear();
    vga::print("Booting Cheesecake Kernel\n");

    print_system_info(boot_info);

    // TODO: Make the Kernel actually preload, like start threads, file-system... only start

    vga::print("Cheesecake Kernel Booted\n");

    loop {
        x86_64::instructions::hlt();
    }
}
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}