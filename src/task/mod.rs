//! Task Management Subsystem
//!
//! Exposes Task Control Blocks, state transitions, and context switching.

pub mod context;
pub mod state;
pub mod task;

pub use context::{switch_context, TaskContext};
pub use state::TaskState;
pub use task::{Task, TaskId, TaskPriority, DEFAULT_STACK_SIZE};
