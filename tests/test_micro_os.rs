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
