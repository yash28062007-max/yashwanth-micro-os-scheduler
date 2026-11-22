//! Yashwanth Micro-OS: Bare-Metal Educational Operating System
//!
//! Architecture: x86_64
//! Bootloader: Limine
//! Specialization: Scheduler Engineering (Dynamic Quantum Round Robin)

#![cfg_attr(not(test), no_std)]

pub mod drivers;
pub mod kernel;
pub mod memory;
pub mod scheduler;
pub mod sync;
pub mod task;
pub mod timer;

/// System halt function for QEMU / bare-metal.
pub fn hlt_loop() -> ! {
    loop {
        #[cfg(any(target_os = "none", feature = "kernel_bin"))]
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
        #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
        {}
    }
}

/// Exits QEMU cleanly using the isa-debug-exit device (port 0xf4).
/// A value written to 0xf4 causes QEMU to exit with code `(val << 1) | 1`.
pub fn exit_qemu(success: bool) {
    let exit_code = if success { 0x10 } else { 0x11 };
    unsafe {
        drivers::port::outb(0xf4, exit_code);
    }
}
