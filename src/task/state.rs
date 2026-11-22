//! Task State Representation
//!
//! Represents the standard lifecycle states of a runnable operating system task.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is newly created or waiting in the Ready queue for CPU time.
    Ready,
    /// Task currently owns the CPU and is actively executing.
    Running,
    /// Task is blocked waiting for an I/O event, timer, or synchronization lock.
    Blocked,
    /// Task has completed execution and is awaiting resource cleanup.
    Terminated,
}

impl TaskState {
    /// Returns a short string representation for structured logging.
    pub const fn as_str(&self) -> &'static str {
        match self {
            TaskState::Ready => "READY",
            TaskState::Running => "RUNNING",
            TaskState::Blocked => "BLOCKED",
            TaskState::Terminated => "TERMINATED",
        }
    }

    /// Verifies whether a state transition from `self` to `target` is valid.
    pub fn can_transition_to(&self, target: TaskState) -> bool {
        match (*self, target) {
            // Valid transitions
            (TaskState::Ready, TaskState::Running) => true,
            (TaskState::Running, TaskState::Ready) => true,       // Preemption or quantum expiry
            (TaskState::Running, TaskState::Blocked) => true,     // Blocked on lock/sleep
            (TaskState::Running, TaskState::Terminated) => true,  // Execution completed
            (TaskState::Blocked, TaskState::Ready) => true,       // Event unblocked
            // Staying in the same state
            (s1, s2) if s1 == s2 => true,
            // All other transitions are invalid
            _ => false,
        }
    }
}
