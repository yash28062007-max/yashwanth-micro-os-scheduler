//! Hardware Device Drivers
//!
//! Provides I/O abstractions for serial console (UART 16550), VGA text mode,
//! and x86 I/O port operations.

pub mod font;
pub mod framebuffer;
pub mod port;
pub mod serial;
pub mod vga;

pub use framebuffer::{init_framebuffer, FRAMEBUFFER_CONSOLE};
pub use port::{inb, inw, io_wait, outb, outw};
pub use serial::{init_serial, SERIAL_PORT};
pub use vga::{clear_screen, Color, VGA_WRITER};
