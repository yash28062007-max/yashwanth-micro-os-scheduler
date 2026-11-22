//! 16550 UART Serial Driver
//!
//! Provides communication through x86 COM1 (0x3F8) serial port.
//! This is the primary output channel for QEMU diagnostics, scheduler traces,
//! and performance analysis logs.

use core::fmt;
use crate::drivers::port::{inb, io_wait, outb};
use crate::sync::SpinLock;

const COM1: u16 = 0x3F8;

/// 16550 UART Serial Port abstraction.
pub struct SerialPort {
    base: u16,
    initialized: bool,
}

impl SerialPort {
    /// Creates a new uninitialized serial port instance.
    pub const fn new(base: u16) -> Self {
        Self {
            base,
            initialized: false,
        }
    }

    /// Initializes the serial port (baud 38400, 8N1, FIFO enabled).
    pub fn init(&mut self) {
        unsafe {
            // Disable all interrupts
            outb(self.base + 1, 0x00);
            io_wait();

            // Enable DLAB (set baud rate divisor)
            outb(self.base + 3, 0x80);
            io_wait();

            // Set divisor to 3 (lo byte) 38400 baud
            outb(self.base + 0, 0x03);
            io_wait();
            outb(self.base + 1, 0x00); // (hi byte)
            io_wait();

            // 8 bits, no parity, one stop bit (8N1), clear DLAB
            outb(self.base + 3, 0x03);
            io_wait();

            // Enable FIFO, clear them, with 14-byte threshold
            outb(self.base + 2, 0xC7);
            io_wait();

            // IRQs enabled, RTS/DSR set
            outb(self.base + 4, 0x0B);
            io_wait();
        }
        self.initialized = true;
    }

    /// Checks if the transmitter FIFO is empty and ready to send.
    fn is_transmit_empty(&self) -> bool {
        unsafe { (inb(self.base + 5) & 0x20) != 0 }
    }

    /// Sends a single byte over the serial port.
    pub fn send_byte(&mut self, byte: u8) {
        if !self.initialized {
            self.init();
        }
        while !self.is_transmit_empty() {
            #[cfg(any(target_os = "none", feature = "kernel_bin"))]
            unsafe {
                core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
            }
            #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
            core::hint::spin_loop();
        }
        unsafe {
            outb(self.base, byte);
        }
    }

    /// Sends a string over the serial port, translating '\n' to '\r\n'.
    pub fn send_str(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send_byte(b'\r');
            }
            self.send_byte(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.send_str(s);
        Ok(())
    }
}

/// Global serial port singleton protected by a spinlock.
pub static SERIAL_PORT: SpinLock<SerialPort> = SpinLock::new(SerialPort::new(COM1));

/// Initializes the primary serial port.
pub fn init_serial() {
    SERIAL_PORT.lock().init();
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    let mut serial = SERIAL_PORT.lock();
    let _ = serial.write_fmt(args);
}

/// Prints formatted text to the serial port.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*))
    };
}

/// Prints formatted text to the serial port followed by a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
