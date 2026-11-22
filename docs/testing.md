# Yashwanth Micro-OS: Testing Strategy & Verification Plan

## 1. Testing Hierarchy

To ensure system reliability without compromising low-level bare-metal fidelity, Yashwanth Micro-OS adopts a three-tiered testing methodology:

```
        +-------------------------------------------------------+
        |  Level 3: Full Integration Testing in QEMU            |
        |  (Boot -> GDT/IDT -> Timer -> Dynamic RR -> Shutdown) |
        +-------------------------------------------------------+
                                   ▲
        +-------------------------------------------------------+
        |  Level 2: Mechanism Verification                      |
        |  (Preemption, Dynamic Quantum, SpinLock, Bump Alloc)  |
        +-------------------------------------------------------+
                                   ▲
        +-------------------------------------------------------+
        |  Level 1: Automated Host-Side Unit Tests              |
        |  (Queue FIFO, State Transitions, Limits, Math Tests)  |
        +-------------------------------------------------------+
```

---

## 2. Distinction Between Host Tests & Kernel Integration Tests

- **Host-Side Tests (`cargo test`)**: Executed on the host architecture using Rust's built-in test framework. Modules are conditionally compiled with `#![cfg(not(target_os = "none"))]` to test core data structures, algorithms, math formulas, and error handling with full assert diagnostics.
- **Kernel Integration Tests (QEMU)**: The real `x86_64-unknown-none` binary boots inside QEMU. Real hardware registers, port I/O, UART serial streams, timer interrupts, and memory layouts are exercised, with output streamed to `traces/qemu_serial.log`.

---

## 3. Official Assignment Test Cases (TC1 - TC6)

### TC1 – Minimum Configuration
- **Objective**: Verify that the scheduler correctly alternates between the minimum required set of two tasks.
- **Implementation**: Two tasks (`T1`, `T2`) spawned with equal bursts and initial quanta.
- **Expected Outcome**: Tasks alternate strictly in round-robin sequence without starvation or deadlock.
- **Status**: **PASS** (Verified in unit test `test_round_robin_preemption_and_switch`).

### TC2 – Multiple Heterogeneous Tasks
- **Objective**: Schedule 4 concurrent tasks with mixed workloads (CPU-bound, interactive yields, synchronization workers, diagnostics).
- **Workload Matrix**:
  - `Task 1`: CPU-Bound (Burst: 18 ticks, Priority: 5)
  - `Task 2`: Interactive Yield (Burst: 10 ticks, Priority: 5)
  - `Task 3`: Synchronization Worker (Burst: 14 ticks, Priority: 8)
  - `Task 4`: Diagnostics Observer (Burst: 6 ticks, Priority: 3)
- **Expected Outcome**: Tasks with higher priorities and shorter bursts complete first; interactive tasks yielding voluntarily re-enter the queue smoothly.
- **Status**: **PASS** (Verified in QEMU boot execution).

### TC3 – Boundary Conditions
- **Objective**: Test system limits and lowest meaningful parameters.
- **Test Scenarios**:
  1. `ReadyQueue` capacity limit (`MAX_TASKS = 16`): Enqueueing 17th task returns `Err(())`.
  2. Single runnable task: Dynamic quantum computes bounded max quantum without division by zero.
  3. Single-tick burst task: Terminates on the exact tick without lingering.
  4. Memory allocator zero-size: Rejects `Layout::from_size_align(0, 8)` with `AllocError::ZeroSizedAllocation`.
- **Status**: **PASS** (Verified in `test_ready_queue_boundary_and_overflow` and `test_bump_allocator_invalid_operations`).

### TC4 – Stress & Scalability Case
- **Objective**: Verify ready queue ring buffer wrap-around and sustained context switching.
- **Implementation**: Continuous enqueue and dequeue cycling across multiple wrap-around intervals.
- **Expected Outcome**: Queue maintains strict FIFO order with zero index drift or memory corruption.
- **Status**: **PASS** (Verified in `test_ready_queue_remove` and extended queue tests).

### TC5 – Failure & Invalid Operation Handling
- **Objective**: Ensure the kernel rejects invalid operations predictably.
- **Scenarios**:
  1. Invalid state transitions (e.g. `Ready` directly to `Terminated` without running): Returns `Err(())`.
  2. Memory allocation exceeding remaining heap capacity: Returns `Err(AllocError::OutOfMemory)`.
  3. Secondary `try_lock()` on an already locked `SpinLock`: Returns `None` without blocking.
- **Status**: **PASS** (Verified in `test_task_state_transitions`, `test_bump_allocator`, and `test_spinlock_try_lock`).

### TC6 – Repeatability & Determinism
- **Objective**: Execute repeated runs of the same workload and compare execution metrics.
- **Metric Measured**: Context switch counts, turnaround times, and scheduling traces.
- **Outcome**: Deterministic match across repeated invocations with zero variance in scheduling decisions.
- **Status**: **PASS** (Captured in `results/performance_metrics.json`).

---

## 4. Test Execution Instructions

### Running Host Automated Test Suite:
```bash
cargo test --verbose
```

### Running QEMU Kernel Integration Test:
```bash
bash scripts/run_qemu.sh
```

### Parsing and Evaluating Execution Traces:
```bash
python3 scripts/analyze_traces.py traces/qemu_serial.log
```
