//! System Tick and Monotonic Time Management
//!
//! Tracks elapsed CPU/timer ticks since kernel boot.

use core::sync::atomic::{AtomicU64, Ordering};

static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

/// Ticks per second (configured frequency, typically 100 Hz = 10ms per tick).
pub const TICKS_PER_SECOND: u64 = 100;
pub const MILLISECONDS_PER_TICK: u64 = 1000 / TICKS_PER_SECOND;

/// Returns current monotonic tick count.
#[inline]
pub fn get_ticks() -> u64 {
    SYSTEM_TICKS.load(Ordering::Relaxed)
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
