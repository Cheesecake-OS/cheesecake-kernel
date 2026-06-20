pub mod buddy;
pub mod heap;

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

pub fn usable_ram_mb(memory_map: &MemoryRegions) -> u64 {
    memory_map
        .iter()
        .filter(|r| r.kind == MemoryRegionKind::Usable)
        .map(|r| r.end - r.start)
        .sum::<u64>()
        / 1024
        / 1024
}
