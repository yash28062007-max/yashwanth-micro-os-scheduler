//! Hardware Port I/O Abstraction for x86_64
//!
//! Provides safe and controlled access to x86 CPU I/O ports using inline assembly.
//! Includes mock implementations when running host-side unit tests.

#[cfg(any(target_os = "none", feature = "kernel_bin"))]
use core::arch::asm;

/// Reads an 8-bit byte from the specified I/O port.
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    #[cfg(any(target_os = "none", feature = "kernel_bin"))]
    {
        let value: u8;
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
        value
    }
    #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
    {
        let _ = port;
        0
    }
}

/// Writes an 8-bit byte to the specified I/O port.
#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    #[cfg(any(target_os = "none", feature = "kernel_bin"))]
    {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
    {
        let _ = (port, value);
    }
}

/// Reads a 16-bit word from the specified I/O port.
#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    #[cfg(any(target_os = "none", feature = "kernel_bin"))]
    {
        let value: u16;
        asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack, preserves_flags));
        value
    }
    #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
    {
        let _ = port;
        0
    }
}

/// Writes a 16-bit word to the specified I/O port.
#[inline]
pub unsafe fn outw(port: u16, value: u16) {
    #[cfg(any(target_os = "none", feature = "kernel_bin"))]
    {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
    {
        let _ = (port, value);
    }
}

/// Brief delay for older hardware I/O operations (writing to unused port 0x80).
#[inline]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}
