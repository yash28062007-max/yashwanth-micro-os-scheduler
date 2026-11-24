//! Automated Host-Side Test Suite for Yashwanth Micro-OS
//!
//! Validates:
//! - Task creation & state transitions
//! - Ready queue FIFO ordering and boundary conditions
//! - Round Robin scheduling & preemption
//! - Dynamic quantum calculation algorithms
//! - Spinlock synchronization primitives
//! - Bump memory allocator edge cases and limits
//! - Monotonic timer tick behaviors

use core::alloc::Layout;
use yashwanth_micro_os::memory::{align_up, AllocError, BumpAllocator};
use yashwanth_micro_os::scheduler::dynamic_quantum::DynamicQuantumPolicy;
use yashwanth_micro_os::scheduler::ready_queue::{ReadyQueue, MAX_TASKS};
use yashwanth_micro_os::scheduler::round_robin::{RoundRobinScheduler, ScheduleAction, SwitchReason};
use yashwanth_micro_os::sync::SpinLock;
use yashwanth_micro_os::task::state::TaskState;
use yashwanth_micro_os::task::task::{Task, TaskId};
use yashwanth_micro_os::timer::{
    consume_pending_ticks, get_hw_interrupt_count, get_ticks, increment_ticks, on_timer_tick_irq,
    reset_ticks, ticks_to_ms,
};

#[test]
fn test_task_creation_and_fields() {
    let task = Task::new(TaskId(1), "TestTask", 5, 20, 6, None);
    assert_eq!(task.id, TaskId(1));
    assert_eq!(task.name, "TestTask");
    assert_eq!(task.state, TaskState::Ready);
    assert_eq!(task.priority, 5);
    assert_eq!(task.burst_remaining, 20);
    assert_eq!(task.current_quantum, 6);
    assert_eq!(task.remaining_quantum, 6);
    assert_eq!(task.total_runtime, 0);
    assert_eq!(task.waiting_time, 0);
}

#[test]
fn test_task_state_transitions() {
    let mut task = Task::new(TaskId(2), "StateTask", 5, 10, 4, None);

    // Ready -> Running: Valid
    assert!(task.transition_to(TaskState::Running).is_ok());
    assert_eq!(task.state, TaskState::Running);

    // Running -> Blocked: Valid
    assert!(task.transition_to(TaskState::Blocked).is_ok());
    assert_eq!(task.state, TaskState::Blocked);

    // Blocked -> Ready: Valid
    assert!(task.transition_to(TaskState::Ready).is_ok());
    assert_eq!(task.state, TaskState::Ready);

    // Ready -> Terminated: Invalid (Task must run before terminating)
    assert!(task.transition_to(TaskState::Terminated).is_err());
    assert_eq!(task.state, TaskState::Ready);

    // Ready -> Running -> Terminated: Valid
    assert!(task.transition_to(TaskState::Running).is_ok());
    assert!(task.transition_to(TaskState::Terminated).is_ok());
    assert_eq!(task.state, TaskState::Terminated);
}

#[test]
fn test_ready_queue_fifo_ordering() {
    let mut queue = ReadyQueue::new();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);

    assert!(queue.enqueue(TaskId(10)).is_ok());
    assert!(queue.enqueue(TaskId(20)).is_ok());
    assert!(queue.enqueue(TaskId(30)).is_ok());
    assert_eq!(queue.len(), 3);

    assert_eq!(queue.peek(), Some(TaskId(10)));
    assert_eq!(queue.dequeue(), Some(TaskId(10)));
    assert_eq!(queue.dequeue(), Some(TaskId(20)));
    assert_eq!(queue.dequeue(), Some(TaskId(30)));
    assert_eq!(queue.dequeue(), None);
    assert!(queue.is_empty());
}

#[test]
fn test_ready_queue_boundary_and_overflow() {
    let mut queue = ReadyQueue::new();
    for i in 0..MAX_TASKS {
        assert!(queue.enqueue(TaskId(i)).is_ok());
    }
    assert!(queue.is_full());
    assert_eq!(queue.len(), MAX_TASKS);

    // Overflow attempt: must reject
    assert!(queue.enqueue(TaskId(999)).is_err());
    assert_eq!(queue.len(), MAX_TASKS);
}

#[test]
fn test_ready_queue_remove() {
    let mut queue = ReadyQueue::new();
    let _ = queue.enqueue(TaskId(1));
    let _ = queue.enqueue(TaskId(2));
    let _ = queue.enqueue(TaskId(3));

    // Remove middle element
    assert!(queue.remove(TaskId(2)));
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.dequeue(), Some(TaskId(1)));
    assert_eq!(queue.dequeue(), Some(TaskId(3)));
    assert_eq!(queue.dequeue(), None);
}

#[test]
fn test_dynamic_quantum_calculation() {
    let policy = DynamicQuantumPolicy::default_config();
    let mut task = Task::new(TaskId(1), "DynaTask", 5, 20, 6, None);

    // With 1 runnable task: quantum should scale up to bound
    let q1 = policy.compute_quantum(&task, 1);
    assert!(q1 >= 10 && q1 <= policy.max_quantum);

    // With 8 runnable tasks: quantum should scale down
    let q8 = policy.compute_quantum(&task, 8);
    assert!(q8 <= q1);
    assert!(q8 >= policy.min_quantum);

    // Voluntary yield behavior (interactive) should produce lower quantum
    task.yield_count = 5;
    task.preempt_count = 1;
    let q_interactive = policy.compute_quantum(&task, 4);

    // CPU-bound behavior (frequent preemption) should produce higher quantum
    task.yield_count = 0;
    task.preempt_count = 5;
    let q_cpubound = policy.compute_quantum(&task, 4);

    assert!(q_interactive <= q_cpubound);
}

#[test]
fn test_round_robin_preemption_and_switch() {
    let mut sched = RoundRobinScheduler::new();
    sched.policy.min_quantum = 2;
    sched.policy.max_quantum = 2;
    let t1 = Task::new(TaskId(1), "T1", 5, 10, 2, None);
    let t2 = Task::new(TaskId(2), "T2", 5, 10, 2, None);

    assert!(sched.spawn_task(t1).is_ok());
    assert!(sched.spawn_task(t2).is_ok());

    // First tick schedules T1
    let act1 = sched.on_timer_tick();
    match act1 {
        ScheduleAction::ContextSwitch { next, .. } => assert_eq!(next, TaskId(1)),
        _ => panic!("Expected context switch to Task 1"),
    }

    // Tick 2: T1 still has remaining quantum
    let act2 = sched.on_timer_tick();
    assert_eq!(act2, ScheduleAction::Continue);

    // Tick 3: T1 quantum expires -> preemption -> switch to T2
    let act3 = sched.on_timer_tick();
    match act3 {
        ScheduleAction::ContextSwitch { prev, next, reason } => {
            assert_eq!(prev, Some(TaskId(1)));
            assert_eq!(next, TaskId(2));
            assert_eq!(reason, SwitchReason::QuantumExpired);
        }
        _ => panic!("Expected preemption context switch to Task 2"),
    }
}

#[test]
fn test_round_robin_task_termination() {
    let mut sched = RoundRobinScheduler::new();
    // Task with burst of 1 tick
    let t1 = Task::new(TaskId(1), "ShortTask", 5, 1, 4, None);
    assert!(sched.spawn_task(t1).is_ok());

    let _ = sched.on_timer_tick(); // Switch to T1
    let act = sched.on_timer_tick(); // Executes burst -> terminates

    match act {
        ScheduleAction::ContextSwitch { .. } | ScheduleAction::Idle => {
            // Task terminated
            let finished_task = sched.get_task(TaskId(1)).unwrap();
            assert_eq!(finished_task.state, TaskState::Terminated);
            assert_eq!(finished_task.burst_remaining, 0);
        }
        _ => panic!("Expected termination"),
    }
}

#[test]
fn test_spinlock_synchronization() {
    let lock = SpinLock::new(42);
    {
        let mut guard = lock.lock();
        assert_eq!(*guard, 42);
        *guard = 99;
    } // Guard drops, lock released

    assert!(!lock.is_locked());
    {
        let guard2 = lock.lock();
        assert_eq!(*guard2, 99);
    }
}

#[test]
fn test_spinlock_try_lock() {
    let lock = SpinLock::new("hello");
    let guard1 = lock.try_lock();
    assert!(guard1.is_some());

    // Secondary try_lock must fail while held
    let guard2 = lock.try_lock();
    assert!(guard2.is_none());

    drop(guard1);
    // After dropping, try_lock succeeds
    let guard3 = lock.try_lock();
    assert!(guard3.is_some());
}

#[test]
fn test_bump_allocator() {
    let mut allocator = BumpAllocator::new();
    let mut backing_buffer = [0u8; 1024];
    unsafe {
        allocator.init(backing_buffer.as_mut_ptr() as usize, 1024);
    }

    assert_eq!(allocator.available_bytes(), 1024);

    let layout1 = Layout::from_size_align(128, 8).unwrap();
    let ptr1 = allocator.allocate(layout1);
    assert!(ptr1.is_ok());
    assert_eq!(allocator.stats().allocations_count, 1);
    assert!(allocator.available_bytes() <= 1024 - 128);

    let layout2 = Layout::from_size_align(256, 16).unwrap();
    let ptr2 = allocator.allocate(layout2);
    assert!(ptr2.is_ok());
    assert_eq!(allocator.stats().allocations_count, 2);

    // Boundary condition: request exceeding remaining space
    let huge_layout = Layout::from_size_align(2048, 8).unwrap();
    assert_eq!(allocator.allocate(huge_layout), Err(AllocError::OutOfMemory));

    // Reset capability
    unsafe {
        allocator.reset();
    }
    assert_eq!(allocator.stats().allocated_bytes, 0);
}

#[test]
fn test_bump_allocator_invalid_operations() {
    let mut allocator = BumpAllocator::new();
    let mut buffer = [0u8; 512];
    unsafe {
        allocator.init(buffer.as_mut_ptr() as usize, 512);
    }

    // Zero-sized allocation
    let zero_layout = Layout::from_size_align(0, 8).unwrap();
    assert_eq!(allocator.allocate(zero_layout), Err(AllocError::ZeroSizedAllocation));
}

#[test]
fn test_align_up_helper() {
    assert_eq!(align_up(0, 8), 0);
    assert_eq!(align_up(1, 8), 8);
    assert_eq!(align_up(8, 8), 8);
    assert_eq!(align_up(9, 8), 16);
    assert_eq!(align_up(15, 16), 16);
    assert_eq!(align_up(16, 16), 16);
}

#[test]
fn test_timer_ticks_and_conversion() {
    reset_ticks();
    assert_eq!(get_ticks(), 0);
    increment_ticks();
    increment_ticks();
    assert_eq!(get_ticks(), 2);
    assert_eq!(ticks_to_ms(10), 100); // 100 Hz = 10 ms/tick
}

#[test]
fn test_hardware_timer_irq_accounting() {
    let initial_hw = get_hw_interrupt_count();
    let initial_ticks = get_ticks();
    let _ = consume_pending_ticks();

    // Simulate 3 hardware IRQ0 pulses
    on_timer_tick_irq();
    on_timer_tick_irq();
    on_timer_tick_irq();

    assert_eq!(get_hw_interrupt_count(), initial_hw + 3);
    assert_eq!(get_ticks(), initial_ticks + 3);
    assert_eq!(consume_pending_ticks(), 3);
    // Consuming pending ticks must reset accumulator to 0
    assert_eq!(consume_pending_ticks(), 0);
}

#[test]
fn test_yield_when_single_runnable_task_avoids_self_switch() {
    let mut sched = RoundRobinScheduler::new();
    let task = Task::new(TaskId(1), "SoloTask", 5, 10, 4, None);
    assert!(sched.spawn_task(task).is_ok());

    // First tick puts SoloTask into Running state
    let init_act = sched.on_timer_tick();
    match init_act {
        ScheduleAction::ContextSwitch { next, reason, .. } => {
            assert_eq!(next, TaskId(1));
            assert_eq!(reason, SwitchReason::InitialSchedule);
        }
        _ => panic!("Expected InitialSchedule"),
    }
    assert_eq!(sched.total_context_switches, 1);

    // Voluntary yield when no other tasks are ready
    let yield_act = sched.yield_current();
    // Must NOT self-switch: should Continue running!
    assert_eq!(yield_act, ScheduleAction::Continue);
    // Context switch count must NOT be incremented
    assert_eq!(sched.total_context_switches, 1);
    // Task yield count must be recorded
    let t = sched.get_task(TaskId(1)).unwrap();
    assert_eq!(t.yield_count, 1);
    assert_eq!(t.state, TaskState::Running);
}

#[test]
fn test_yield_with_multiple_tasks_performs_cooperative_switch() {
    let mut sched = RoundRobinScheduler::new();
    let t1 = Task::new(TaskId(1), "TaskA", 5, 10, 4, None);
    let t2 = Task::new(TaskId(2), "TaskB", 5, 10, 4, None);
    assert!(sched.spawn_task(t1).is_ok());
    assert!(sched.spawn_task(t2).is_ok());

    // Schedule TaskA
    let _ = sched.on_timer_tick();
    assert_eq!(sched.current_task_id(), Some(TaskId(1)));

    // TaskA voluntarily yields CPU
    let act = sched.yield_current();
    match act {
        ScheduleAction::ContextSwitch { prev, next, reason } => {
            assert_eq!(prev, Some(TaskId(1)));
            assert_eq!(next, TaskId(2));
            assert_eq!(reason, SwitchReason::TaskYielded);
        }
        _ => panic!("Expected ContextSwitch with TaskYielded reason"),
    }

    assert_eq!(sched.current_task_id(), Some(TaskId(2)));
    assert_eq!(sched.get_task(TaskId(1)).unwrap().state, TaskState::Ready);
    assert_eq!(sched.get_task(TaskId(2)).unwrap().state, TaskState::Running);
    assert_eq!(sched.get_task(TaskId(1)).unwrap().yield_count, 1);
}

#[test]
fn test_task_response_time_calculation() {
    let mut sched = RoundRobinScheduler::new();
    let t1 = Task::new(TaskId(1), "TaskA", 5, 10, 4, None);
    let t2 = Task::new(TaskId(2), "TaskB", 5, 10, 4, None);
    assert!(sched.spawn_task(t1).is_ok());
    assert!(sched.spawn_task(t2).is_ok());

    // Initially, neither task has been scheduled
    assert_eq!(sched.get_task(TaskId(1)).unwrap().response_time(), 0);
    assert_eq!(sched.get_task(TaskId(2)).unwrap().response_time(), 0);

    // Tick 1: Task 1 is scheduled for the first time at total_ticks = 1
    let _ = sched.on_timer_tick();
    assert_eq!(sched.get_task(TaskId(1)).unwrap().first_run_time, Some(1));
    assert_eq!(sched.get_task(TaskId(1)).unwrap().response_time(), 1);

    // Task 1 yields CPU to Task 2
    let _ = sched.yield_current();
    assert_eq!(sched.current_task_id(), Some(TaskId(2)));
    assert_eq!(sched.get_task(TaskId(2)).unwrap().first_run_time, Some(1));
    assert_eq!(sched.get_task(TaskId(2)).unwrap().response_time(), 1);
}

#[test]
fn test_task_runtime_based_yield_determinism() {
    let mut sched = RoundRobinScheduler::new();
    let t1 = Task::new(TaskId(1), "CPU", 5, 20, 5, None);
    let t2 = Task::new(TaskId(2), "Interactive", 5, 10, 10, None);
    assert!(sched.spawn_task(t1).is_ok());
    assert!(sched.spawn_task(t2).is_ok());

    // Schedule Task 1, then Task 1 yields to let Task 2 run
    let _ = sched.on_timer_tick();
    let _ = sched.yield_current();

    // Now Task 2 is the running task
    assert_eq!(sched.current_task_id(), Some(TaskId(2)));
    let mut last_yield_runtime: u32 = u32::MAX;

    // Simulate Task 2 runtime-based voluntary yield check (every 3 runtime ticks)
    let mut yield_observed_at_runtime_3 = false;
    for _ in 0..5 {
        let _ = sched.on_timer_tick();
        if let Some(curr) = sched.current_task_id() {
            if curr == TaskId(2) {
                let task = sched.get_task(curr).unwrap();
                if task.total_runtime > 0 && task.total_runtime % 3 == 0 && task.total_runtime != last_yield_runtime {
                    last_yield_runtime = task.total_runtime;
                    let _ = last_yield_runtime;
                    assert_eq!(task.total_runtime, 3);
                    yield_observed_at_runtime_3 = true;
                    let action = sched.yield_current();
                    // Yielding with Task 1 in ready queue must switch to Task 1
                    match action {
                        ScheduleAction::ContextSwitch { next, reason, .. } => {
                            assert_eq!(next, TaskId(1));
                            assert_eq!(reason, SwitchReason::TaskYielded);
                        }
                        _ => panic!("Expected ContextSwitch to Task 1 on yield"),
                    }
                    break;
                }
            }
        }
    }

    assert!(yield_observed_at_runtime_3, "Task 2 must yield exactly at total_runtime == 3");
}

#[test]
fn test_performance_metrics_math_integrity() {
    let mut sched = RoundRobinScheduler::new();
    let t1 = Task::new(TaskId(1), "T1", 5, 6, 4, None);
    let t2 = Task::new(TaskId(2), "T2", 5, 4, 4, None);
    assert!(sched.spawn_task(t1).is_ok());
    assert!(sched.spawn_task(t2).is_ok());

    // Run until all tasks finish
    let mut ticks = 0;
    while sched.runnable_task_count() > 0 && ticks < 100 {
        let _ = sched.on_timer_tick();
        ticks += 1;
    }

    assert_eq!(sched.completed_tasks_count, 2);

    for id in 1..=2 {
        let task = sched.get_task(TaskId(id)).unwrap();
        // Turnaround Time must equal Runtime + Waiting Time
        assert_eq!(
            task.turnaround_time(),
            (task.total_runtime as u64) + (task.waiting_time as u64),
            "Mathematical invariant violated: Turnaround != Runtime + Waiting for Task {}",
            id
        );
        // Response time must be <= Waiting Time
        assert!(
            task.response_time() <= (task.waiting_time as u64),
            "Response time cannot exceed total waiting time"
        );
    }
}

// ==============================================================================
// DEDICATED MANDATORY ASSIGNMENT TEST CASES (TC1 - TC6)
// ==============================================================================

/// Helper to calculate Jain's Fairness Index over a slice of numbers.
fn jains_fairness_index(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 1.0;
    }
    let sum: f64 = values.iter().sum();
    let sum_sq: f64 = values.iter().map(|v| v * v).sum();
    let n = values.len() as f64;
    if sum_sq == 0.0 {
        return 1.0;
    }
    (sum * sum) / (n * sum_sq)
}

/// TC1: Minimum Configuration
/// - Objective: Verify that the scheduler correctly alternates between the minimum
///   required set of two tasks (T1, T2) with equal burst and quantum.
/// - Verification: Preemption on quantum expiration, round-robin alternation,
///   zero starvation, exact context switch count.
#[test]
fn test_tc1_minimum_configuration() {
    let mut sched = RoundRobinScheduler::new();
    sched.policy.min_quantum = 3;
    sched.policy.max_quantum = 3;

    // Two identical tasks: Burst = 6 ticks, Quantum = 3 ticks
    let t1 = Task::new(TaskId(1), "TC1-Task-1", 5, 6, 3, None);
    let t2 = Task::new(TaskId(2), "TC1-Task-2", 5, 6, 3, None);

    assert!(sched.spawn_task(t1).is_ok());
    assert!(sched.spawn_task(t2).is_ok());
    assert_eq!(sched.runnable_task_count(), 2);

    let mut switch_sequence = Vec::new();

    // Step through execution until both tasks complete
    let mut ticks = 0;
    while sched.runnable_task_count() > 0 && ticks < 50 {
        let action = sched.on_timer_tick();
        ticks += 1;
        if let ScheduleAction::ContextSwitch { prev, next, reason } = action {
            switch_sequence.push((ticks, prev, next, reason));
        }
    }

    // Verification 1: Both tasks completed
    assert_eq!(sched.completed_tasks_count, 2, "Both tasks must terminate");
    assert_eq!(sched.runnable_task_count(), 0);

    // Verification 2: Switch sequence must strictly alternate
    // Tick 1: INIT -> T1 (InitialSchedule)
    // Tick 4: T1 -> T2 (QuantumExpired: T1 ran 3 ticks)
    // Tick 7: T2 -> T1 (QuantumExpired: T2 ran 3 ticks)
    // Tick 10: T1 -> T2 (TaskTerminated: T1 finished 6 ticks total)
    assert_eq!(switch_sequence.len(), 4, "Must have exactly 4 context switches");

    assert_eq!(switch_sequence[0].1, None);
    assert_eq!(switch_sequence[0].2, TaskId(1));
    assert_eq!(switch_sequence[0].3, SwitchReason::InitialSchedule);

    assert_eq!(switch_sequence[1].1, Some(TaskId(1)));
    assert_eq!(switch_sequence[1].2, TaskId(2));
    assert_eq!(switch_sequence[1].3, SwitchReason::QuantumExpired);

    assert_eq!(switch_sequence[2].1, Some(TaskId(2)));
    assert_eq!(switch_sequence[2].2, TaskId(1));
    assert_eq!(switch_sequence[2].3, SwitchReason::QuantumExpired);

    assert_eq!(switch_sequence[3].1, Some(TaskId(1)));
    assert_eq!(switch_sequence[3].2, TaskId(2));
    assert_eq!(switch_sequence[3].3, SwitchReason::TaskTerminated);

    // Verification 3: Metrics integrity
    let task1 = sched.get_task(TaskId(1)).unwrap();
    let task2 = sched.get_task(TaskId(2)).unwrap();
    assert_eq!(task1.total_runtime, 6);
    assert_eq!(task2.total_runtime, 6);
    assert_eq!(task1.burst_remaining, 0);
    assert_eq!(task2.burst_remaining, 0);

    // Fairness across identical tasks: identical runtimes -> 1.0
    let fairness = jains_fairness_index(&[task1.total_runtime as f64, task2.total_runtime as f64]);
    assert!((fairness - 1.0).abs() < 1e-6, "Jain's index for equal tasks must be 1.0");
}

/// TC2: Multiple Heterogeneous Tasks
/// - Objective: Schedule 4 concurrent tasks with mixed workloads (CPU-bound, interactive yields,
///   synchronization worker, diagnostics observer).
/// - Verification: Dynamic quantum adaptation, voluntary yield transitions, waiting, turnaround,
///   response times, context switch breakdown, Jain's fairness index, and starvation prevention.
#[test]
fn test_tc2_heterogeneous_tasks() {
    let mut sched = RoundRobinScheduler::new();

    // 4 Heterogeneous tasks matching CA-6 specifications
    let t1 = Task::new(TaskId(1), "Task-1-CPUBound", 5, 18, 6, None);
    let t2 = Task::new(TaskId(2), "Task-2-Interactive", 5, 10, 6, None);
    let t3 = Task::new(TaskId(3), "Task-3-SyncWorker", 8, 14, 6, None);
    let t4 = Task::new(TaskId(4), "Task-4-Diagnostics", 3, 6, 6, None);

    assert!(sched.spawn_task(t1).is_ok());
    assert!(sched.spawn_task(t2).is_ok());
    assert!(sched.spawn_task(t3).is_ok());
    assert!(sched.spawn_task(t4).is_ok());

    let mut last_yield_runtime: u32 = u32::MAX;
    let mut total_switches = 0;
    let mut switch_reasons = Vec::new();

    // Execution loop simulating bare-metal main loop
    while sched.runnable_task_count() > 0 && sched.total_ticks < 100 {
        let tick_action = sched.on_timer_tick();

        let action = if tick_action == ScheduleAction::Continue {
            if let Some(curr) = sched.current_task_id() {
                let should_yield = if curr == TaskId(2) {
                    if let Some(task) = sched.get_task(curr) {
                        task.total_runtime > 0
                            && task.total_runtime % 3 == 0
                            && task.total_runtime != last_yield_runtime
                    } else {
                        false
                    }
                } else {
                    false
                };

                if should_yield {
                    if let Some(task) = sched.get_task(curr) {
                        last_yield_runtime = task.total_runtime;
                    }
                    sched.yield_current()
                } else {
                    tick_action
                }
            } else {
                tick_action
            }
        } else {
            tick_action
        };

        if let ScheduleAction::ContextSwitch { reason, .. } = action {
            total_switches += 1;
            switch_reasons.push(reason);
        }
    }

    // 1. Completion & Starvation Prevention
    assert_eq!(sched.completed_tasks_count, 4, "All 4 tasks must complete without starvation");
    assert_eq!(sched.total_ticks, 49, "Total scheduling ticks must equal 49 (18+10+14+6 burst + 1 dispatch)");
    assert_eq!(total_switches, 6, "Total context switches must equal 6");

    // 2. Context Switch Reason Breakdown
    let initial_count = switch_reasons.iter().filter(|&&r| r == SwitchReason::InitialSchedule).count();
    let quantum_count = switch_reasons.iter().filter(|&&r| r == SwitchReason::QuantumExpired).count();
    let yield_count = switch_reasons.iter().filter(|&&r| r == SwitchReason::TaskYielded).count();
    let term_count = switch_reasons.iter().filter(|&&r| r == SwitchReason::TaskTerminated).count();

    assert_eq!(initial_count, 1, "Must have 1 InitialSchedule");
    assert_eq!(quantum_count, 1, "Must have 1 QuantumExpired");
    assert_eq!(yield_count, 1, "Must have 1 TaskYielded");
    assert_eq!(term_count, 3, "Must have 3 TaskTerminated");

    // 3. Per-Task Metrics Invariant Checks
    let t1 = sched.get_task(TaskId(1)).unwrap();
    let t2 = sched.get_task(TaskId(2)).unwrap();
    let t3 = sched.get_task(TaskId(3)).unwrap();
    let t4 = sched.get_task(TaskId(4)).unwrap();

    assert_eq!(t1.total_runtime, 18);
    assert_eq!(t2.total_runtime, 10);
    assert_eq!(t3.total_runtime, 14);
    assert_eq!(t4.total_runtime, 6);

    assert_eq!(t1.waiting_time, 24);
    assert_eq!(t2.waiting_time, 39);
    assert_eq!(t3.waiting_time, 13);
    assert_eq!(t4.waiting_time, 27);

    assert_eq!(t1.turnaround_time(), 42);
    assert_eq!(t2.turnaround_time(), 49);
    assert_eq!(t3.turnaround_time(), 27);
    assert_eq!(t4.turnaround_time(), 33);

    assert_eq!(t1.response_time(), 1);
    assert_eq!(t2.response_time(), 10);
    assert_eq!(t3.response_time(), 13);
    assert_eq!(t4.response_time(), 27);

    // 4. Jain's Fairness Index
    let runtimes = [t1.total_runtime as f64, t2.total_runtime as f64, t3.total_runtime as f64, t4.total_runtime as f64];
    let fairness_runtime = jains_fairness_index(&runtimes);
    assert!((fairness_runtime - 0.8780).abs() < 0.001, "Jain's index on runtime must be ~0.8780");

    let turnarounds = [t1.turnaround_time() as f64, t2.turnaround_time() as f64, t3.turnaround_time() as f64, t4.turnaround_time() as f64];
    let fairness_turnaround = jains_fairness_index(&turnarounds);
    assert!(fairness_turnaround > 0.90, "Jain's index on turnaround must be > 0.90");
}

/// TC3: Boundary Conditions
/// - Objective: Validate limits, edge conditions, zero parameters, and single-task execution.
#[test]
fn test_tc3_boundary_conditions() {
    // 1. ReadyQueue capacity boundary (MAX_TASKS = 16)
    let mut queue = ReadyQueue::new();
    for i in 0..MAX_TASKS {
        assert!(queue.enqueue(TaskId(i)).is_ok());
    }
    assert!(queue.is_full());
    assert_eq!(queue.enqueue(TaskId(99)), Err(()), "17th task must be rejected");

    // 2. Empty scheduler tick
    let mut empty_sched = RoundRobinScheduler::new();
    assert_eq!(empty_sched.on_timer_tick(), ScheduleAction::Idle, "Empty scheduler must return Idle");

    // 3. Single-task voluntary yield: no self-switch, continue
    let mut solo_sched = RoundRobinScheduler::new();
    let solo = Task::new(TaskId(1), "Solo", 5, 10, 4, None);
    assert!(solo_sched.spawn_task(solo).is_ok());
    let _ = solo_sched.on_timer_tick(); // Switch to Solo
    let yield_act = solo_sched.yield_current();
    assert_eq!(yield_act, ScheduleAction::Continue, "Single task yielding must Continue");
    assert_eq!(solo_sched.total_context_switches, 1, "Context switch count must not increment");

    // 4. Single-tick burst task
    let mut short_sched = RoundRobinScheduler::new();
    let short_task = Task::new(TaskId(1), "1Tick", 5, 1, 4, None);
    assert!(short_sched.spawn_task(short_task).is_ok());
    let _ = short_sched.on_timer_tick(); // Dispatches
    let term_act = short_sched.on_timer_tick(); // Executes 1 tick and finishes
    assert!(matches!(term_act, ScheduleAction::ContextSwitch { .. } | ScheduleAction::Idle));
    assert_eq!(short_sched.get_task(TaskId(1)).unwrap().state, TaskState::Terminated);

    // 5. Memory Allocator Zero-Size Request
    let mut alloc = BumpAllocator::new();
    let mut buf = [0u8; 256];
    unsafe { alloc.init(buf.as_mut_ptr() as usize, 256); }
    let zero_layout = Layout::from_size_align(0, 8).unwrap();
    assert_eq!(alloc.allocate(zero_layout), Err(AllocError::ZeroSizedAllocation));
}

/// TC4: Stress & Scalability
/// - Objective: Validate ring buffer wrap-around and sustained context switching over multiple iterations.
#[test]
fn test_tc4_stress_and_scalability() {
    // 1. Ring buffer wrap-around stress: 500 enqueue/dequeue cycles
    let mut queue = ReadyQueue::new();
    for cycle in 0..500 {
        assert!(queue.enqueue(TaskId(cycle % MAX_TASKS)).is_ok());
        let dequeued = queue.dequeue();
        assert_eq!(dequeued, Some(TaskId(cycle % MAX_TASKS)));
        assert_eq!(queue.len(), 0);
    }

    // 2. Scalability: Spawn and complete full capacity (16 tasks, IDs 0..16) concurrently
    let mut sched = RoundRobinScheduler::new();
    sched.policy.min_quantum = 2;
    sched.policy.max_quantum = 4;

    for id in 0..MAX_TASKS {
        let burst: u32 = (id as u32 % 3) + 2; // 2, 3, or 4 ticks burst
        let prio = (id as u8 % 5) + 3;
        let t = Task::new(TaskId(id), "CapacityTask", prio, burst, 2, None);
        assert!(sched.spawn_task(t).is_ok());
    }
    assert_eq!(sched.runnable_task_count(), MAX_TASKS);

    // Run until all 16 tasks complete
    let mut limit = 500;
    while sched.runnable_task_count() > 0 && limit > 0 {
        let _ = sched.on_timer_tick();
        limit -= 1;
    }

    assert_eq!(sched.completed_tasks_count, 16, "All 16 capacity tasks must complete cleanly");
    assert_eq!(sched.runnable_task_count(), 0);
}

/// TC5: Failure & Invalid Operation Handling
/// - Objective: Verify kernel rejects illegal operations predictably without panicking or corrupting state.
#[test]
fn test_tc5_failure_and_invalid_operations() {
    // 1. Illegal Task State Transitions
    let mut task = Task::new(TaskId(1), "FailTest", 5, 10, 4, None);
    assert_eq!(task.state, TaskState::Ready);

    // Ready -> Terminated directly (must fail)
    assert!(task.transition_to(TaskState::Terminated).is_err());
    assert_eq!(task.state, TaskState::Ready);

    // Ready -> Running -> Blocked
    assert!(task.transition_to(TaskState::Running).is_ok());
    assert!(task.transition_to(TaskState::Blocked).is_ok());

    // Blocked -> Running directly (must fail: must go through Ready)
    assert!(task.transition_to(TaskState::Running).is_err());
    assert_eq!(task.state, TaskState::Blocked);

    // 2. Memory Allocator Heap Exhaustion (OOM)
    let mut alloc = BumpAllocator::new();
    let mut buf = [0u8; 512];
    unsafe { alloc.init(buf.as_mut_ptr() as usize, 512); }
    let oversized = Layout::from_size_align(1024, 8).unwrap();
    assert_eq!(alloc.allocate(oversized), Err(AllocError::OutOfMemory));
    // Verify allocator remains intact
    assert_eq!(alloc.available_bytes(), 512);

    // 3. SpinLock Contention & Non-blocking try_lock
    let lock = SpinLock::new(12345);
    let guard1 = lock.try_lock();
    assert!(guard1.is_some());
    assert!(lock.try_lock().is_none(), "Secondary lock attempt must return None without blocking");
    drop(guard1);
    assert!(lock.try_lock().is_some(), "Lock must be acquirable again after drop");
}

/// TC6: Repeatability & Determinism
/// - Objective: Verify that identical workloads executed in separate runs yield
///   bit-for-bit identical metrics and context switch counts.
#[test]
fn test_tc6_repeatability_and_determinism() {
    fn run_simulation() -> (u64, u32, Vec<u64>, Vec<u64>, Vec<u64>) {
        let mut sched = RoundRobinScheduler::new();
        let t1 = Task::new(TaskId(1), "Task-1-CPUBound", 5, 18, 6, None);
        let t2 = Task::new(TaskId(2), "Task-2-Interactive", 5, 10, 6, None);
        let t3 = Task::new(TaskId(3), "Task-3-SyncWorker", 8, 14, 6, None);
        let t4 = Task::new(TaskId(4), "Task-4-Diagnostics", 3, 6, 6, None);

        let _ = sched.spawn_task(t1);
        let _ = sched.spawn_task(t2);
        let _ = sched.spawn_task(t3);
        let _ = sched.spawn_task(t4);

        let mut last_yield_runtime: u32 = u32::MAX;
        let mut switches = 0;

        while sched.runnable_task_count() > 0 && sched.total_ticks < 100 {
            let tick_action = sched.on_timer_tick();
            let action = if tick_action == ScheduleAction::Continue {
                if let Some(curr) = sched.current_task_id() {
                    let should_yield = if curr == TaskId(2) {
                        if let Some(task) = sched.get_task(curr) {
                            task.total_runtime > 0
                                && task.total_runtime % 3 == 0
                                && task.total_runtime != last_yield_runtime
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if should_yield {
                        if let Some(task) = sched.get_task(curr) {
                            last_yield_runtime = task.total_runtime;
                        }
                        sched.yield_current()
                    } else {
                        tick_action
                    }
                } else {
                    tick_action
                }
            } else {
                tick_action
            };

            if let ScheduleAction::ContextSwitch { .. } = action {
                switches += 1;
            }
        }

        let waitings: Vec<u64> = (1..=4).map(|id| sched.get_task(TaskId(id)).unwrap().waiting_time as u64).collect();
        let turnarounds: Vec<u64> = (1..=4).map(|id| sched.get_task(TaskId(id)).unwrap().turnaround_time()).collect();
        let responses: Vec<u64> = (1..=4).map(|id| sched.get_task(TaskId(id)).unwrap().response_time()).collect();

        (sched.total_ticks, switches, waitings, turnarounds, responses)
    }

    let run_a = run_simulation();
    let run_b = run_simulation();

    // Verify 100% determinism between Run A and Run B
    assert_eq!(run_a.0, run_b.0, "Total ticks must be identical across runs");
    assert_eq!(run_a.1, run_b.1, "Total context switches must be identical across runs");
    assert_eq!(run_a.2, run_b.2, "Waiting times must be identical across runs");
    assert_eq!(run_a.3, run_b.3, "Turnaround times must be identical across runs");
    assert_eq!(run_a.4, run_b.4, "Response times must be identical across runs");
}
