//! Intel 8254 Programmable Interval Timer (PIT) Driver
//!
//! Generates periodic hardware interrupts (IRQ0) for preemptive scheduling
//! and time slice measurement.

use crate::drivers::port::{io_wait, outb};

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_MODE_COMMAND: u16 = 0x43;
const PIT_BASE_FREQUENCY: u32 = 1193182; // 1.193182 MHz

/// Initializes the 8254 PIT to generate timer ticks at `frequency_hz`.
pub unsafe fn init_pit(frequency_hz: u32) {
    let divisor = (PIT_BASE_FREQUENCY / frequency_hz) as u16;

    // Command: Channel 0, Access mode lobyte/hibyte, Mode 3 (Square Wave Generator), 16-bit binary
    outb(PIT_MODE_COMMAND, 0x36);
    io_wait();

    // Write low byte then high byte of divisor
    outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
    io_wait();
    outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);
    io_wait();
}
