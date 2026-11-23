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

#[cfg(any(target_os = "none", feature = "kernel_bin"))]
extern "C" {
    fn timer_interrupt_entry();
    fn default_irq_entry();
    fn exception_no_err_entry();
    fn exception_err_entry();
}

/// High-level Rust timer interrupt handler called directly by `timer_interrupt_entry`.
/// Completely lock-free: increments atomic tick counters and sends PIC EOI.
#[no_mangle]
pub extern "C" fn timer_interrupt_handler() {
    crate::timer::on_timer_tick_irq();
    unsafe {
        send_eoi(0);
    }
}

#[cfg(any(target_os = "none", feature = "kernel_bin"))]
core::arch::global_asm!(
    r#"
.global timer_interrupt_entry
.type timer_interrupt_entry, @function
timer_interrupt_entry:
    # 1. Save all scratch and callee-saved registers + caller RBP
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push rbx
    push r12
    push r13
    push r14
    push r15
    push rbp

    # 2. Dynamic 16-byte stack alignment
    mov rbp, rsp
    and rsp, -16

    # 3. Call Rust timer interrupt handler
    call timer_interrupt_handler

    # 4. Restore stack pointer and all registers in reverse
    mov rsp, rbp
    pop rbp
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax

    # 5. Return from interrupt
    iretq

.global default_irq_entry
.type default_irq_entry, @function
default_irq_entry:
    push rax
    mov al, 0x20
    out 0x20, al
    pop rax
    iretq

.global exception_no_err_entry
.type exception_no_err_entry, @function
exception_no_err_entry:
    cli
    hlt
    jmp exception_no_err_entry

.global exception_err_entry
.type exception_err_entry, @function
exception_err_entry:
    cli
    add rsp, 8
    hlt
    jmp exception_err_entry
"#
);

/// Initializes the IDT, unmasks IRQ0 on 8259 PIC, and loads the IDTR into the CPU.
pub unsafe fn init_idt() {
    #[cfg(any(target_os = "none", feature = "kernel_bin"))]
    {
        remap_pic(0x20, 0x28);

        // Unmask IRQ0 (PIT timer: bit 0 = 0), mask all other Master PIC IRQs (0xFE)
        outb(PIC1_DATA, 0xFE);
        io_wait();
        // Mask all Slave PIC IRQs (0xFF)
        outb(PIC2_DATA, 0xFF);
        io_wait();

        // 1. Install exception handlers without error code for vectors 0..31
        for vec in 0..32 {
            IDT[vec].set_handler(exception_no_err_entry as *const () as usize);
        }

        // 2. Install exception handlers with hardware error code:
        // Vectors: 8 (#DF), 10 (#TS), 11 (#NP), 12 (#SS), 13 (#GP), 14 (#PF), 17 (#AC), 21 (#CP), 29 (#VC), 30 (#SX)
        const ERR_VECTORS: [usize; 10] = [8, 10, 11, 12, 13, 14, 17, 21, 29, 30];
        for &vec in &ERR_VECTORS {
            IDT[vec].set_handler(exception_err_entry as *const () as usize);
        }

        // 3. Install default IRQ handlers for vectors 33..255
        for vec in 33..256 {
            IDT[vec].set_handler(default_irq_entry as *const () as usize);
        }

        // 4. Install dedicated PIT timer interrupt handler on vector 32 (IRQ0)
        IDT[32].set_handler(timer_interrupt_entry as *const () as usize);

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
