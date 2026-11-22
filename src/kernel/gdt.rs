//! Global Descriptor Table (GDT) Setup for x86_64
//!
//! Provides standard 64-bit flat segment selectors for kernel code (CS) and data (DS/SS).

#[allow(dead_code)]
#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

#[allow(dead_code)]
#[repr(C, align(8))]
struct Gdt {
    entries: [u64; 3],
}

#[allow(dead_code)]
static mut GDT: Gdt = Gdt {
    entries: [
        0,                  // Null descriptor
        0x00af9a000000ffff, // Kernel Code 64-bit (Ring 0, Exec/Read, Conforming=0, DPL=0, P=1, L=1)
        0x00cf92000000ffff, // Kernel Data 64-bit (Ring 0, Read/Write, DPL=0, P=1)
    ],
};

/// Loads the Global Descriptor Table.
pub unsafe fn init_gdt() {
    #[cfg(any(target_os = "none", feature = "kernel_bin"))]
    {
        let descriptor = GdtDescriptor {
            size: (core::mem::size_of::<Gdt>() - 1) as u16,
            offset: core::ptr::addr_of!(GDT) as u64,
        };

        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &descriptor,
            options(readonly, nostack, preserves_flags)
        );

        // Reload segment registers
        core::arch::asm!(
            "push 0x08",
            "lea {tmp}, [2f + rip]",
            "push {tmp}",
            "retfq",
            "2:",
            "mov {data_sel:x}, 0x10",
            "mov ds, {data_sel:x}",
            "mov es, {data_sel:x}",
            "mov ss, {data_sel:x}",
            "mov fs, {data_sel:x}",
            "mov gs, {data_sel:x}",
            tmp = out(reg) _,
            data_sel = out(reg) _,
            options(nomem)
        );
    }
}
