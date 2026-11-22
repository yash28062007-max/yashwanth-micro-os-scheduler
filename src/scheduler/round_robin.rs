//! Round Robin Scheduler with Dynamic Quantum Specialization
//!
//! Implements preemptive Round Robin CPU scheduling, task lifecycle management,
//! dynamic quantum calculation, and kernel execution diagnostics.

use crate::scheduler::dynamic_quantum::DynamicQuantumPolicy;
use crate::scheduler::ready_queue::{ReadyQueue, MAX_TASKS};
use crate::task::{Task, TaskId, TaskState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchReason {
    QuantumExpired,
    TaskYielded,
    TaskTerminated,
    TaskBlocked,
    InitialSchedule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleAction {
    Continue,
    ContextSwitch {
        prev: Option<TaskId>,
        next: TaskId,
        reason: SwitchReason,
    },
    Idle,
}

/// Round Robin Scheduler engine with dynamic quantum support.
pub struct RoundRobinScheduler {
    tasks: [Option<Task>; MAX_TASKS],
    ready_queue: ReadyQueue,
    current_task_id: Option<TaskId>,
    pub policy: DynamicQuantumPolicy,
    pub total_context_switches: u64,
    pub total_ticks: u64,
    pub completed_tasks_count: u32,
}

impl RoundRobinScheduler {
    /// Creates a new scheduler instance.
    pub const fn new() -> Self {
        const INIT_TASK: Option<Task> = None;
        Self {
            tasks: [INIT_TASK; MAX_TASKS],
            ready_queue: ReadyQueue::new(),
            current_task_id: None,
            policy: DynamicQuantumPolicy::default_config(),
            total_context_switches: 0,
            total_ticks: 0,
            completed_tasks_count: 0,
        }
    }

    /// Spawns a new task into the scheduler and enqueues it into the ready queue.
    pub fn spawn_task(&mut self, mut task: Task) -> Result<TaskId, ()> {
        let slot = task.id.0;
        if slot >= MAX_TASKS || self.tasks[slot].is_some() {
            return Err(());
        }

        task.arrival_time = self.total_ticks;
        let id = task.id;
        self.tasks[slot] = Some(task);
        self.ready_queue.enqueue(id)?;
        Ok(id)
    }

    /// Returns a reference to a task by ID.
    pub fn get_task(&self, id: TaskId) -> Option<&Task> {
        if id.0 < MAX_TASKS {
            self.tasks[id.0].as_ref()
        } else {
            None
        }
    }

    /// Returns a mutable reference to a task by ID.
    pub fn get_task_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        if id.0 < MAX_TASKS {
            self.tasks[id.0].as_mut()
        } else {
            None
        }
    }

    /// Returns the currently running task ID.
    pub fn current_task_id(&self) -> Option<TaskId> {
        self.current_task_id
    }

    /// Advances scheduler by one timer tick.
    /// Updates running task runtime, decrements remaining quantum and burst,
    /// and triggers a scheduling decision if quantum expires or task finishes.
    pub fn on_timer_tick(&mut self) -> ScheduleAction {
        self.total_ticks += 1;

        // Increment waiting time for all tasks currently in the Ready Queue
        for slot in 0..MAX_TASKS {
            if let Some(task) = &mut self.tasks[slot] {
                if task.state == TaskState::Ready {
                    task.waiting_time += 1;
                }
            }
        }

        if let Some(curr_id) = self.current_task_id {
            let task = match self.tasks[curr_id.0].as_mut() {
                Some(t) => t,
                None => return self.schedule_next(SwitchReason::InitialSchedule),
            };

            task.total_runtime += 1;
            if task.burst_remaining > 0 {
                task.burst_remaining -= 1;
            }

            // Check if task completed its work
            if task.burst_remaining == 0 {
                task.finish_time = self.total_ticks;
                let _ = task.transition_to(TaskState::Terminated);
                self.completed_tasks_count += 1;
                return self.schedule_next(SwitchReason::TaskTerminated);
            }

            // Decrement remaining time quantum
            if task.remaining_quantum > 0 {
                task.remaining_quantum -= 1;
            }

            // Quantum exhausted -> preemption!
            if task.remaining_quantum == 0 {
                task.preempt_count += 1;
                return self.schedule_next(SwitchReason::QuantumExpired);
            }

            ScheduleAction::Continue
        } else {
            // No current task running, pick one from ready queue
            self.schedule_next(SwitchReason::InitialSchedule)
        }
    }

    /// Handles a voluntary task yield.
    pub fn yield_current(&mut self) -> ScheduleAction {
        if let Some(curr_id) = self.current_task_id {
            if let Some(task) = self.tasks[curr_id.0].as_mut() {
                task.yield_count += 1;
            }
            self.schedule_next(SwitchReason::TaskYielded)
        } else {
            ScheduleAction::Idle
        }
    }

    /// Selects the next task to run from the Ready Queue using Round Robin.
    pub fn schedule_next(&mut self, reason: SwitchReason) -> ScheduleAction {
        let prev_id = self.current_task_id;

        // Requeue previous task if it was preempted or yielded and still runnable
        if let Some(prev) = prev_id {
            if let Some(task) = self.tasks[prev.0].as_mut() {
                if task.state == TaskState::Running {
                    let _ = task.transition_to(TaskState::Ready);
                    let _ = self.ready_queue.enqueue(prev);
                }
            }
        }

        // Dequeue next ready task
        if let Some(next_id) = self.ready_queue.dequeue() {
            let runnable_count = self.ready_queue.len() + 1;

            if let Some(next_task) = self.tasks[next_id.0].as_mut() {
                let _ = next_task.transition_to(TaskState::Running);
                next_task.context_switch_count += 1;

                // SPECIALIZATION: Dynamically compute the time quantum!
                let dynamic_quantum = self.policy.compute_quantum(next_task, runnable_count);
                next_task.current_quantum = dynamic_quantum;
                next_task.remaining_quantum = dynamic_quantum;
            }

            self.current_task_id = Some(next_id);
            self.total_context_switches += 1;

            ScheduleAction::ContextSwitch {
                prev: prev_id,
                next: next_id,
                reason,
            }
        } else {
            self.current_task_id = None;
            ScheduleAction::Idle
        }
    }

    /// Returns the number of runnable tasks currently in system.
    pub fn runnable_task_count(&self) -> usize {
        let running = if self.current_task_id.is_some() { 1 } else { 0 };
        self.ready_queue.len() + running
    }
}
