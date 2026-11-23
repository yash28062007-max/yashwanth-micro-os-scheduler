//! System Tick and Monotonic Time Management
//!
//! Tracks elapsed CPU/timer ticks since kernel boot.

use core::sync::atomic::{AtomicU64, Ordering};

static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);
static HW_INTERRUPT_COUNT: AtomicU64 = AtomicU64::new(0);
static PENDING_TICKS: AtomicU64 = AtomicU64::new(0);

/// Ticks per second (configured frequency, typically 100 Hz = 10ms per tick).
pub const TICKS_PER_SECOND: u64 = 100;
pub const MILLISECONDS_PER_TICK: u64 = 1000 / TICKS_PER_SECOND;

/// Lock-free timer interrupt callback called directly by the hardware IRQ0 ISR.
/// Increments the monotonic tick, hardware interrupt counter, and accumulated pending ticks.
#[inline]
pub fn on_timer_tick_irq() {
    SYSTEM_TICKS.fetch_add(1, Ordering::Relaxed);
    HW_INTERRUPT_COUNT.fetch_add(1, Ordering::Relaxed);
    PENDING_TICKS.fetch_add(1, Ordering::Release);
}

/// Returns current monotonic tick count.
#[inline]
pub fn get_ticks() -> u64 {
    SYSTEM_TICKS.load(Ordering::Relaxed)
}

/// Returns the total number of hardware timer interrupts (IRQ0) serviced.
#[inline]
pub fn get_hw_interrupt_count() -> u64 {
    HW_INTERRUPT_COUNT.load(Ordering::Relaxed)
}

/// Consumes and clears any accumulated hardware timer ticks since last check.
#[inline]
pub fn consume_pending_ticks() -> u64 {
    PENDING_TICKS.swap(0, Ordering::AcqRel)
}

/// Increments the monotonic tick count and returns the updated value.
#[inline]
pub fn increment_ticks() -> u64 {
    SYSTEM_TICKS.fetch_add(1, Ordering::Relaxed) + 1
}

/// Converts ticks into elapsed milliseconds.
#[inline]
pub fn ticks_to_ms(ticks: u64) -> u64 {
    ticks * MILLISECONDS_PER_TICK
}

/// Resets the tick counter (used for benchmarking/isolated tests).
#[inline]
pub fn reset_ticks() {
    SYSTEM_TICKS.store(0, Ordering::Relaxed);
}
