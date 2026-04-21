pub mod buddy;
pub mod heap;

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub fn usable_ram_mb(memory_map: &MemoryMap) -> u64 {
    memory_map.iter()
        .filter(|r| r.region_type == MemoryRegionType::Usable)
        .map(|r| r.range.end_addr() - r.range.start_addr())
        .sum::<u64>() / 1024 / 1024
}