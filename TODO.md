# Cheesecake Kernel - TODO

## Tasks / Scheduler
- [ ] Multi-threading groundwork
- [ ] Multi-threading
- [ ] User space implementation

## Storage
- [ ] Detect drives (AHCI or virtio-blk under QEMU)
- [ ] Read raw sectors
- [ ] FAT32 parser
- [ ] VFS abstraction layer

## Shell
- [ ] Simple command parser in kernel space
- [ ] Commands: `help`, `clear`, `mem`, `cpuinfo`, `halt`
- [ ] Eventually move to userspace process once syscalls + memory isolation exist

## Power / ACPI
- [ ] ACPI table parsing (RSDP → RSDT/XSDT → FACP)
- [ ] `\_S5` shutdown via ACPI PM1a control register
- [ ] Reboot via keyboard controller (port 0x64) or ACPI reset register

## Cleanup
- [ ] Fix `static_mut_refs` warnings in `cpu.rs` and `heap.rs` before Rust 2024 makes them errors
- [ ] Remove unused imports and dead fields (`base` in `BuddyAllocator`, `NonZeroUsize` in `task.rs`)

## Readme
- [ ] Update README.md with building instructions
