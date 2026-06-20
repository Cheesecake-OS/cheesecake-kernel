use crate::serial_println;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::Mutex;

const PAGE_SIZE: usize = 4096;
const MAX_ORDER: usize = 11;
const MAX_PAGES: usize = 1 << 20; // 1M pages

struct FreeList {
    heads: [Option<usize>; MAX_ORDER],
    next: [usize; MAX_PAGES],
    free_count: [usize; MAX_ORDER],
}

impl FreeList {
    const fn new() -> Self {
        FreeList {
            heads: [None; MAX_ORDER],
            next: [usize::MAX; MAX_PAGES],
            free_count: [0; MAX_ORDER],
        }
    }
}

pub struct BuddyAllocator {
    inner: Mutex<FreeList>,
    base: usize,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        BuddyAllocator {
            inner: Mutex::new(FreeList::new()),
            base: 0,
        }
    }

    pub fn add_region(&self, start: usize, end: usize) {
        let mut list = self.inner.lock();
        let mut addr = (start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1); // align up
        let end = end & !(PAGE_SIZE - 1); // align down

        while addr + PAGE_SIZE <= end {
            let mut order = MAX_ORDER - 1;
            loop {
                let size = PAGE_SIZE << order;
                if addr % size == 0 && addr + size <= end {
                    break;
                }
                if order == 0 {
                    break;
                }
                order -= 1;
            }
            let pfn = addr / PAGE_SIZE;
            let old_head = list.heads[order];
            list.next[pfn] = old_head.unwrap_or(usize::MAX);
            list.heads[order] = Some(pfn);
            list.free_count[order] += 1;
            addr += PAGE_SIZE << order;
        }
    }

    pub fn alloc(&self, order: usize) -> Option<usize> {
        let mut list = self.inner.lock();
        self.alloc_inner(&mut list, order)
    }

    fn alloc_inner(&self, list: &mut FreeList, order: usize) -> Option<usize> {
        if order >= MAX_ORDER {
            return None;
        }

        if let Some(pfn) = list.heads[order] {
            let next = list.next[pfn];
            list.heads[order] = if next == usize::MAX { None } else { Some(next) };
            list.free_count[order] -= 1;
            return Some(pfn * PAGE_SIZE);
        }

        let addr = self.alloc_inner(list, order + 1)?;
        let pfn = addr / PAGE_SIZE;
        let buddy_pfn = pfn + (1 << order); // push buddy back

        let old_head = list.heads[order];
        list.next[buddy_pfn] = old_head.unwrap_or(usize::MAX);
        list.heads[order] = Some(buddy_pfn);
        list.free_count[order] += 1;

        Some(addr)
    }

    pub fn free(&self, addr: usize, order: usize) {
        let mut list = self.inner.lock();
        let mut pfn = addr / PAGE_SIZE;
        let mut ord = order;

        loop {
            if ord >= MAX_ORDER - 1 {
                break;
            }

            let buddy_pfn = pfn ^ (1 << ord); // XOR to find buddy

            let mut prev: Option<usize> = None;
            let mut cur = list.heads[ord];
            let mut found = false;

            while let Some(c) = cur {
                if c == buddy_pfn {
                    if let Some(p) = prev {
                        list.next[p] = list.next[c];
                    } else {
                        let next = list.next[c];
                        list.heads[ord] = if next == usize::MAX { None } else { Some(next) };
                    }
                    list.free_count[ord] -= 1;
                    found = true;
                    break;
                }
                prev = cur;
                cur = if list.next[c] == usize::MAX {
                    None
                } else {
                    Some(list.next[c])
                };
            }

            if !found {
                break;
            }

            pfn = pfn.min(buddy_pfn);
            ord += 1;
        }

        let old_head = list.heads[ord];
        list.next[pfn] = old_head.unwrap_or(usize::MAX);
        list.heads[ord] = Some(pfn);
        list.free_count[ord] += 1;
    }

    pub fn free_pages(&self, order: usize) -> usize {
        self.inner.lock().free_count[order]
    }

    pub fn print_stats(&self) {
        let list = self.inner.lock();
        serial_println!("=== Buddy Allocator Stats ===");
        for ord in 0..MAX_ORDER {
            if list.free_count[ord] > 0 {
                serial_println!(
                    "  order {:2}: {} free blocks ({} KiB each)",
                    ord,
                    list.free_count[ord],
                    (PAGE_SIZE << ord) / 1024
                );
            }
        }
    }
}

pub static BUDDY: BuddyAllocator = BuddyAllocator::new();

pub fn init(memory_map: &MemoryRegions) {
    for region in memory_map.iter() {
        if region.kind == MemoryRegionKind::Usable {
            BUDDY.add_region(region.start as usize, region.end as usize);
        }
    }
    serial_println!("Buddy allocator initialized.");
    BUDDY.print_stats();
}
