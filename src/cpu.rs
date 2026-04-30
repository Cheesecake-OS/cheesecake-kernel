use core::arch::x86_64::__cpuid;
use crate::serial_println;

#[derive(Debug)]
pub struct CpuInfo {
    pub vendor: &'static str,
    pub brand: [u8; 48],
    pub has_sse: bool,
    pub has_sse2: bool,
    pub has_avx: bool,
    pub has_avx2: bool,
    pub physical_address_bits: u8,
}

static mut CPU_VENDOR_BUF: [u8; 12] = [0u8; 12];

impl CpuInfo {
    pub fn collect() -> Self {
        let vendor = unsafe {
            let r = __cpuid(0);
            CPU_VENDOR_BUF[0..4].copy_from_slice(&r.ebx.to_le_bytes());
            CPU_VENDOR_BUF[4..8].copy_from_slice(&r.edx.to_le_bytes());
            CPU_VENDOR_BUF[8..12].copy_from_slice(&r.ecx.to_le_bytes());
            core::str::from_utf8_unchecked(&CPU_VENDOR_BUF)
        };

        let mut brand = [0u8; 48];
        unsafe {
            let regs = [__cpuid(0x80000002), __cpuid(0x80000003), __cpuid(0x80000004)];
            for (i, r) in regs.iter().enumerate() {
                let off = i * 16;
                brand[off..off+4].copy_from_slice(&r.eax.to_le_bytes());
                brand[off+4..off+8].copy_from_slice(&r.ebx.to_le_bytes());
                brand[off+8..off+12].copy_from_slice(&r.ecx.to_le_bytes());
                brand[off+12..off+16].copy_from_slice(&r.edx.to_le_bytes());
            }
        }

        let (has_sse, has_sse2, has_avx) = unsafe {
            let r = __cpuid(1);
            (
                r.edx & (1 << 25) != 0,
                r.edx & (1 << 26) != 0,
                r.ecx & (1 << 28) != 0,
            )
        };

        let has_avx2 = unsafe {
            let r = __cpuid(7);
            r.ebx & (1 << 5) != 0
        };

        let physical_address_bits = unsafe {
            let r = __cpuid(0x80000008);
            (r.eax & 0xFF) as u8
        };

        CpuInfo { vendor, brand, has_sse, has_sse2, has_avx, has_avx2, physical_address_bits }
    }

    pub fn brand_str(&self) -> &str {
        let s = core::str::from_utf8(&self.brand).unwrap_or("Unknown");
        s.trim_matches('\0').trim()
    }

    pub fn print_info(&self) {
        serial_println!("CPU Vendor : {}", self.vendor);
        serial_println!("CPU Brand  : {}", self.brand_str());
        serial_println!("SSE={} SSE2={} AVX={} AVX2={}", self.has_sse, self.has_sse2, self.has_avx, self.has_avx2);
        serial_println!("Physical addr bits: {}", self.physical_address_bits);
    }
}
