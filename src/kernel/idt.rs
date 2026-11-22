//! Interrupt Descriptor Table (IDT) and 8259 PIC Handling
//!
//! Provides hardware interrupt handling and CPU exception dispatching.

use crate::drivers::port::{inb, io_wait, outb};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;
const PIC_EOI: u8 = 0x20;

#[derive(Copy, Clone, Default)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: usize) {
        self.offset_low = handler as u16;
        self.selector = 0x08; // Kernel code segment selector
        self.ist = 0;
        self.type_attr = 0x8E; // Present, Ring 0, 64-bit Interrupt Gate
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.zero = 0;
    }
}

#[allow(dead_code)]
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

#[allow(dead_code)]
static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];

/// Remaps the 8259 PIC interrupt vectors from 0x08-0x0F to 0x20-0x27 (Master)
/// and 0x70-0x77 to 0x28-0x2F (Slave) to prevent clashes with x86 CPU exceptions.
pub unsafe fn remap_pic(offset1: u8, offset2: u8) {
    let a1 = inb(PIC1_DATA);
    let a2 = inb(PIC2_DATA);

    outb(PIC1_COMMAND, ICW1_INIT);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT);
    io_wait();

    outb(PIC1_DATA, offset1);
    io_wait();
    outb(PIC2_DATA, offset2);
    io_wait();

    outb(PIC1_DATA, 4);
    io_wait();
    outb(PIC2_DATA, 2);
    io_wait();

    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    outb(PIC1_DATA, a1);
    outb(PIC2_DATA, a2);
}

/// Sends End of Interrupt (EOI) to PIC controllers.
pub unsafe fn send_eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC2_COMMAND, PIC_EOI);
    }
    outb(PIC1_COMMAND, PIC_EOI);
}

/// Initializes the IDT and loads it into the CPU.
pub unsafe fn init_idt() {
    #[cfg(any(target_os = "none", feature = "kernel_bin"))]
    {
        remap_pic(0x20, 0x28);

        let idt_ptr = IdtPointer {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };

        core::arch::asm!(
            "lidt [{}]",
            in(reg) &idt_ptr,
            options(readonly, nostack, preserves_flags)
        );
    }
}
