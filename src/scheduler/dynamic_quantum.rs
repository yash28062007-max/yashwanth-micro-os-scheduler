//! Specialization: Dynamic Time Quantum Policy
//!
//! Provides an adaptive CPU time slice recalculation algorithm for Round Robin scheduling.
//!
//! # Problem Statement:
//! Traditional static Round Robin uses a fixed time quantum `Q`.
//! - If `Q` is too large, interactive tasks suffer high response latency (approaching FCFS).
//! - If `Q` is too small, high context-switching frequency wastes CPU cycles.
//! - Static `Q` cannot adapt to varying system loads (queue length) or task burst behaviors.
//!
//! # Dynamic Quantum Adaptation Formula:
//! 1. Target Cycle Constraint:
//!    T_cycle = N_runnable * Q_avg <= T_target
//!    => Q_load = clamp(T_target / max(1, N_runnable), Q_min, Q_max)
//!
//! 2. Workload / Behavior Adjustment:
//!    - Tasks with high voluntary yield ratio receive an interactive bonus (lower quantum, rapid return).
//!    - CPU-bound tasks that exhaust their slices receive longer quanta to maximize CPU throughput.
//!    - Priority scaling: Q_final = Q_load * (priority / P_base).

use crate::task::Task;

#[derive(Debug, Clone, Copy)]
pub struct DynamicQuantumPolicy {
    pub min_quantum: u32,
    pub max_quantum: u32,
    pub base_quantum: u32,
    pub target_cycle_ticks: u32,
}

impl DynamicQuantumPolicy {
    /// Creates a default dynamic quantum configuration.
    pub const fn default_config() -> Self {
        Self {
            min_quantum: 2,
            max_quantum: 16,
            base_quantum: 6,
            target_cycle_ticks: 36,
        }
    }

    /// Calculates an adaptive time quantum for `task` given current `runnable_count`.
    pub fn compute_quantum(&self, task: &Task, runnable_count: usize) -> u32 {
        let n = if runnable_count == 0 { 1 } else { runnable_count as u32 };

        // 1. Target cycle scaling: bound worst-case latency across all runnable tasks
        let load_quantum = (self.target_cycle_ticks / n).clamp(self.min_quantum, self.max_quantum);

        // 2. Behavioral adjustment based on yield vs preemption ratio
        let behavior_factor: f32 = if task.preempt_count > 0 && task.yield_count == 0 {
            // Pure CPU-bound workload: scale up quantum to reduce context switches
            1.25
        } else if task.yield_count > task.preempt_count {
            // Interactive / I/O-bound workload: lower quantum for rapid turnaround
            0.75
        } else {
            1.0
        };

        // 3. Priority weight adjustment (Priority 1..10, Normal = 5)
        let priority_factor: f32 = (task.priority as f32) / 5.0;

        let raw_quantum = ((load_quantum as f32) * behavior_factor * priority_factor) as u32;

        // 4. Strict clamping within safe boundaries
        raw_quantum.clamp(self.min_quantum, self.max_quantum)
    }
}
