use bootloader::BootInfo;
use crate::cpu::CpuInfo;
use crate::mm;
use crate::serial_println;
use crate::vga;

pub struct Kernel {
    pub cpu: CpuInfo,
    pub ram_mb: u64,
}

impl Kernel {
    pub fn init(boot_info: &'static BootInfo) -> Self {
        let cpu = CpuInfo::collect();
        let ram_mb = mm::usable_ram_mb(&boot_info.memory_map);


        mm::buddy::init(&boot_info.memory_map);
        mm::heap::init();

        cpu.print_info();
        serial_println!("Usable RAM: {} MB", ram_mb);

        Kernel { cpu, ram_mb }
    }

    pub fn print_banner(&self) {
        vga::print(" _____ _                                 _\n");
        vga::print("|  __ \\ |                               | |\n");
        vga::print("| /  \\/ |__   ___  ___  ___  ___  __ _| | _____ \n");
        vga::print("| |   | '_ \\ / _ \\/ _ \\/ __|/ _ \\/ _` | |/ / _ \\\n");
        vga::print("| \\__/\\ | | |  __/  __/\\__ \\  __/ (_| |   <  __/\n");
        vga::print(" \\____/_| |_|\\___|\\___|\\___/\\___|\\__,_|_|\\_\\___|\n\n");
    }
}