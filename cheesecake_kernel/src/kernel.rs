use crate::cpu::CpuInfo;
use crate::mm;
use crate::serial_println;
use crate::tasks;
use crate::vga;
use bootloader_api::BootInfo;

pub struct Kernel {
    pub cpu: CpuInfo,
    pub ram_mb: u64,
}

impl Kernel {
    pub fn init(boot_info: &'static BootInfo) -> Self {
        // Get some info
        let cpu = CpuInfo::collect();
        let ram_mb = mm::usable_ram_mb(&boot_info.memory_regions);
        serial_println!("[ OK ] System info collected");

        // Initialize mm
        mm::buddy::init(&boot_info.memory_regions);
        mm::heap::init();
        serial_println!("[ OK ] Memory manager initalized");

        // Initialize scheduler
        tasks::scheduler::SCHEDULER.lock().init();
        serial_println!("[ OK ] Scheduler initalized");

        cpu.print_info();
        serial_println!("Usable RAM: {} MB", ram_mb);

        Kernel { cpu, ram_mb }
    }

    pub fn print_banner(&self) {
        vga::print("   _____ _                                   _        \n");
        vga::print("  / ____| |                                 | |       \n");
        vga::print(" | |    | |__   ___  ___  ___  ___  ___ __ _| | _____ \n");
        vga::print(" | |    | '_ \\ / _ \\/ _ \\/ __|/ _ \\/ __/ _` | |/ / _ \\\n");
        vga::print(" | |____| | | |  __/  __/\\__ \\  __/ (_| (_| |   <  __/\n");
        vga::print("  \\_____|_| |_|\\___|\\___||___/\\___|\\___\\__,_|_|\\_\\___|\n");
        vga::print("                                                      \n");
        vga::print("                 Cheesecake Kernel\n\n");
    }
}
