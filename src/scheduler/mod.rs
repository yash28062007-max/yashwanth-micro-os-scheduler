//! Scheduler Subsystem
//!
//! Provides the Ready Queue, Dynamic Quantum Policy, and Round Robin Scheduler engine.

pub mod dynamic_quantum;
pub mod ready_queue;
pub mod round_robin;

pub use dynamic_quantum::DynamicQuantumPolicy;
pub use ready_queue::{ReadyQueue, MAX_TASKS};
pub use round_robin::{RoundRobinScheduler, ScheduleAction, SwitchReason};

use crate::sync::SpinLock;

/// Global Kernel Scheduler singleton protected by a SpinLock.
pub static SCHEDULER: SpinLock<RoundRobinScheduler> = SpinLock::new(RoundRobinScheduler::new());
