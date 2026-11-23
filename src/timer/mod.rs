//! Timer Subsystem
//!
//! Provides monotonic tick accounting, PIT hardware timer management,
//! and time slice monitoring.

pub mod pit;
pub mod tick;

pub use pit::init_pit;
pub use tick::{
    consume_pending_ticks, get_hw_interrupt_count, get_ticks, increment_ticks, on_timer_tick_irq,
    reset_ticks, ticks_to_ms, MILLISECONDS_PER_TICK, TICKS_PER_SECOND,
};
