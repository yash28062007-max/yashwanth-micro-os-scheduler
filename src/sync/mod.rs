//! Synchronization Subsystem
//!
//! Provides kernel concurrency primitives.

pub mod spinlock;

pub use spinlock::{SpinLock, SpinLockGuard};
