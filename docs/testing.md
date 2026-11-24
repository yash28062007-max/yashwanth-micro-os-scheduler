# Yashwanth Micro-OS: Testing Strategy & Verification Plan

## 1. Testing Hierarchy

To ensure system reliability without compromising low-level bare-metal fidelity, Yashwanth Micro-OS adopts a three-tiered testing methodology:

```
        +-------------------------------------------------------+
        |  Level 3: Full Integration Testing in QEMU & VBox     |
        |  (Boot -> GDT/IDT -> Timer IRQ -> Dynamic RR -> Park) |
        +-------------------------------------------------------+
                                   ▲
        +-------------------------------------------------------+
        |  Level 2: Mechanism Verification & Algorithmic Audit  |
        |  (Preemption, Dynamic Quantum, 64-Test Python Suite)  |
        +-------------------------------------------------------+
                                   ▲
        +-------------------------------------------------------+
        |  Level 1: Automated Host-Side Unit Tests (26 tests)   |
        |  (TC1-TC6, FIFO, States, Limits, Single/Multi-Yields) |
        +-------------------------------------------------------+
```

---

## 2. Distinction Between Host Tests & Kernel Integration Tests

- **Host-Side Tests (`cargo test`)**: Executed on the host architecture using Rust's built-in test framework. Modules are conditionally compiled with `#![cfg(not(target_os = "none"))]` to test core data structures, algorithms, math formulas, yield behaviors, and error handling with full assert diagnostics.
- **Kernel Integration Tests (QEMU & VirtualBox)**: The real `x86_64-unknown-none` binary boots inside QEMU and VirtualBox. Real hardware registers, port I/O, UART serial streams, physical 8254 PIT timer interrupts, and linear framebuffer memory are exercised.

---

## 3. Official Assignment Test Cases (TC1 - TC6)

### TC1 – Minimum Configuration
- **Objective**: Verify that the scheduler correctly alternates between the minimum required set of two tasks.
- **Implementation**: Two tasks (`T1`, `T2`) spawned with equal bursts (6 ticks) and initial quanta (3 ticks).
- **Expected Outcome**: Tasks alternate strictly in round-robin sequence without starvation or deadlock.
- **Verification Evidence**: Automated unit test `test_tc1_minimum_configuration` verifies that Task 1 runs for its quantum (3 ticks), gets preempted with `SwitchReason::QuantumExpired`, switches to Task 2, alternates back, and both terminate cleanly with 4 context switches and Jain's index of 1.0000.
- **Status**: **PASS (Verified on Host & Emulator)**

### TC2 – Multiple Heterogeneous Tasks
- **Objective**: Schedule 4 concurrent tasks with mixed workloads (CPU-bound, interactive yields, synchronization workers, diagnostics).
- **Workload Matrix**:
  - `Task 1`: CPU-Bound (Burst: 18 ticks, Priority: 5)
  - `Task 2`: Interactive Yield (Burst: 10 ticks, Priority: 5)
  - `Task 3`: Synchronization Worker (Burst: 14 ticks, Priority: 8)
  - `Task 4`: Diagnostics Observer (Burst: 6 ticks, Priority: 3)
- **Expected Outcome**: Higher priority tasks receive larger quanta; interactive tasks yielding voluntarily re-enter the queue smoothly; all tasks finish with zero starvation.
- **Verification Evidence**: Automated unit test `test_tc2_heterogeneous_tasks` and live boot in QEMU and VirtualBox completed all 4 tasks with 6 context switches, average turnaround of 37 ticks, average waiting of 25 ticks, average response of 12 ticks, and Jain's Fairness Index of 0.8780.
- **Status**: **PASS (Verified on Bare-Metal QEMU & VirtualBox)**

### TC3 – Boundary Conditions
- **Objective**: Test system limits, single-task execution, and lowest meaningful parameters.
- **Test Scenarios**:
  1. `ReadyQueue` capacity limit (`MAX_TASKS = 16`): Enqueueing a 17th task returns `Err(())`.
  2. Empty scheduler tick: Returns `ScheduleAction::Idle`.
  3. Single runnable task voluntary yield: Returns `ScheduleAction::Continue` without an unnecessary self-switch.
  4. Single-tick burst task: Finishes in 1 tick and terminates cleanly.
  5. Memory allocator zero-size: Rejects `Layout::from_size_align(0, 8)` with `AllocError::ZeroSizedAllocation`.
- **Verification Evidence**: Automated unit test `test_tc3_boundary_conditions`.
- **Status**: **PASS (Verified on Host)**

### TC4 – Stress & Scalability Case
- **Objective**: Verify ready queue ring buffer wrap-around and sustained context switching.
- **Implementation**: 500 continuous enqueue and dequeue cycling across multiple wrap-around intervals, followed by scheduling full capacity (16 tasks) to completion.
- **Expected Outcome**: Queue maintains strict FIFO order with zero index drift, zero memory leaks, and all 16 tasks complete.
- **Verification Evidence**: Automated unit test `test_tc4_stress_and_scalability`.
- **Status**: **PASS (Verified on Host)**

### TC5 – Failure & Invalid Operation Handling
- **Objective**: Ensure the kernel rejects invalid operations predictably without crashing.
- **Scenarios**:
  1. Invalid state transitions (e.g. `Ready` directly to `Terminated` without running, or `Blocked` directly to `Running`): Returns `Err(())`.
  2. Memory allocation exceeding remaining heap capacity: Returns `Err(AllocError::OutOfMemory)` while preserving allocator pointers.
  3. Secondary `try_lock()` on an already locked `SpinLock`: Returns `None` without blocking or deadlocking.
- **Verification Evidence**: Automated unit test `test_tc5_failure_and_invalid_operations`.
- **Status**: **PASS (Verified on Host)**

### TC6 – Repeatability & Determinism
- **Objective**: Execute repeated runs of the same workload and compare execution metrics.
- **Metric Measured**: Context switch counts (6), scheduling ticks, turnaround, waiting, and response times.
- **Outcome**: 100% bit-for-bit deterministic match between independent runs.
- **Verification Evidence**: Automated unit test `test_tc6_repeatability_and_determinism`, `traces/qemu_serial.log`, and `results/performance_metrics.json`.
- **Status**: **PASS (Verified across QEMU & VirtualBox)**

---

## 4. Complete List of 26 Host Unit Tests

```text
test test_align_up_helper ............................................ ok
test test_bump_allocator ............................................. ok
test test_bump_allocator_invalid_operations .......................... ok
test test_dynamic_quantum_calculation ................................ ok
test test_hardware_timer_irq_accounting .............................. ok
test test_performance_metrics_math_integrity ........................ ok
test test_ready_queue_boundary_and_overflow .......................... ok
test test_ready_queue_fifo_ordering .................................. ok
test test_ready_queue_remove ......................................... ok
test test_round_robin_preemption_and_switch .......................... ok
test test_round_robin_task_termination ............................... ok
test test_spinlock_synchronization ................................... ok
test test_spinlock_try_lock .......................................... ok
test test_task_creation_and_fields ................................... ok
test test_task_response_time_calculation ............................. ok
test test_task_runtime_based_yield_determinism ....................... ok
test test_task_state_transitions ..................................... ok
test test_tc1_minimum_configuration .................................. ok
test test_tc2_heterogeneous_tasks .................................... ok
test test_tc3_boundary_conditions .................................... ok
test test_tc4_stress_and_scalability ................................. ok
test test_tc5_failure_and_invalid_operations ......................... ok
test test_tc6_repeatability_and_determinism .......................... ok
test test_timer_ticks_and_conversion ................................. ok
test test_yield_when_single_runnable_task_avoids_self_switch ......... ok
test test_yield_with_multiple_tasks_performs_cooperative_switch ...... ok

Result: 26 passed; 0 failed; 0 ignored (100% Passing)
```

---

## 5. Structural & Algorithmic Audit (64 Automated Python Checks)

The test script `tests/verify_implementation.py` performs deep code audits:
- **Rust Source Syntax Balance:** 30 source files verified for balanced braces and formatting.
- **ReadyQueue FIFO Ring Buffer:** 16 enqueue/dequeue operations, overflow rejection, slot re-use, wrap-around.
- **Dynamic Quantum Formulas:** Low load expansion ($Q \ge 12$), high load contraction ($Q \le 6$), behavior factor differentiation ($Q_{\text{interactive}} \le Q_{\text{cpubound}}$).
- **Bump Allocator & Alignment Logic:** Power-of-two alignment, heap fit checks, out-of-memory boundary checks.

**Result: 64 PASSED; 0 FAILED**
