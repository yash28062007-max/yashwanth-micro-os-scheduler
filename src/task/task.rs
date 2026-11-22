//! Task Representation and Control Block
//!
//! Encapsulates state, execution budget, metrics, and resources for an
//! independent schedulable kernel task.

use crate::task::context::TaskContext;
use crate::task::state::TaskState;

pub const DEFAULT_STACK_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Low = 1,
    Normal = 5,
    High = 10,
}

/// Task Control Block (TCB) for the Micro-OS.
pub struct Task {
    pub id: TaskId,
    pub name: &'static str,
    pub state: TaskState,
    pub priority: u8,
    pub burst_remaining: u32,
    pub total_runtime: u32,
    pub waiting_time: u32,
    pub arrival_time: u64,
    pub finish_time: u64,
    pub current_quantum: u32,
    pub remaining_quantum: u32,
    pub context_switch_count: u32,
    pub yield_count: u32,
    pub preempt_count: u32,
    pub context: TaskContext,
    pub stack: [u8; DEFAULT_STACK_SIZE],
}

impl Task {
    /// Creates a new controlled task with specified execution parameters.
    pub fn new(
        id: TaskId,
        name: &'static str,
        priority: u8,
        burst_ticks: u32,
        initial_quantum: u32,
        entry: Option<fn()>,
    ) -> Self {
        let mut task = Self {
            id,
            name,
            state: TaskState::Ready,
            priority,
            burst_remaining: burst_ticks,
            total_runtime: 0,
            waiting_time: 0,
            arrival_time: 0,
            finish_time: 0,
            current_quantum: initial_quantum,
            remaining_quantum: initial_quantum,
            context_switch_count: 0,
            yield_count: 0,
            preempt_count: 0,
            context: TaskContext::empty(),
            stack: [0; DEFAULT_STACK_SIZE],
        };

        if let Some(entry_fn) = entry {
            let stack_top = task.stack.as_ptr() as usize + DEFAULT_STACK_SIZE;
            task.context.init_stack(stack_top, entry_fn as usize);
        }

        task
    }

    /// Transitions task to a new state if valid.
    pub fn transition_to(&mut self, next_state: TaskState) -> Result<(), ()> {
        if self.state.can_transition_to(next_state) {
            self.state = next_state;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Calculates turnaround time (finish_time - arrival_time).
    pub fn turnaround_time(&self) -> u64 {
        if self.finish_time >= self.arrival_time {
            self.finish_time - self.arrival_time
        } else {
            0
        }
    }

    /// Checks if the task has completed its workload.
    pub fn is_completed(&self) -> bool {
        self.burst_remaining == 0
    }
}
