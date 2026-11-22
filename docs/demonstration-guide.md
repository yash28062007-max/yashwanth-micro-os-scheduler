# Yashwanth Micro-OS: Demonstration & Evaluation Guide

This guide provides an end-to-end evaluation script for instructors, examiners, and evaluators to reproduce all tests and observe the bare-metal kernel and companion scheduler simulator.

---

## 1. Environment & Prerequisites Verification

Run the following command to verify that all required host tools are installed:
```bash
rustc --version
qemu-system-x86_64 --version
xorriso --version
python3 --version
```

**Expected Result:**
- `rustc` $\ge 1.75$ (or `rustc 1.100.0`) with target `x86_64-unknown-none` installed.
- `qemu-system-x86_64` $\ge 7.0$ (with `-cpu max` support).
- `xorriso` $\ge 1.5$.
- `python3` $\ge 3.10$.

---

## 2. Step 1: Automated Unit Test Suite

Execute the host-side unit test suite:
```bash
cargo test --verbose
```

**Expected Output:**
```text
running 14 tests
test test_align_up_helper ... ok
test test_bump_allocator ... ok
test test_bump_allocator_invalid_operations ... ok
test test_dynamic_quantum_calculation ... ok
test test_ready_queue_boundary_and_overflow ... ok
test test_ready_queue_remove ... ok
test test_round_robin_task_termination ... ok
test test_ready_queue_fifo_ordering ... ok
test test_round_robin_preemption_and_switch ... ok
test test_spinlock_synchronization ... ok
test test_spinlock_try_lock ... ok
test test_task_creation_and_fields ... ok
test test_task_state_transitions ... ok
test test_timer_ticks_and_conversion ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## 3. Step 2: Algorithmic Verification Suite

Execute the structural and boundary assertion suite:
```bash
python3 tests/verify_implementation.py
```

**Expected Output:**
- 28 source file syntax checks passing.
- 13 ReadyQueue FIFO & boundary checks passing (rejects 17th task overflow).
- 4 dynamic quantum formula checks passing (load expansion, contraction, boundary clamping).
- 6 bump allocator alignment checks passing.
- **Summary:** `VERIFICATION SUMMARY: 62 PASSED, 0 FAILED`.

---

## 4. Step 3: Bare-Metal Kernel Binary Compilation

Compile the freestanding ELF executable targeting `x86_64-unknown-none`:
```bash
cargo build --target x86_64-unknown-none --release --features kernel_bin
```

**Expected Output:**
- Clean compilation without warnings or errors.
- Binary generated at `target/x86_64-unknown-none/release/yashwanth_micro_os` (statically linked ELF64, entry point `0xffffffff80001060`).

---

## 5. Step 4: Bootable Hybrid ISO Creation

Package the kernel and Limine bootloader into a bootable hybrid ISO:
```bash
bash scripts/build_iso.sh
```

**Expected Output:**
- `xorriso` produces `build/yashwanth_micro_os.iso` (~2.5 MiB).
- `limine bios-install` stamps Stage 1 & Stage 2 MBR/GPT boot records:
  ```text
  Limine BIOS stages installed successfully!
  === ISO Build Complete: build/yashwanth_micro_os.iso ===
  ```

---

## 6. Step 5: Bare-Metal QEMU Boot & Scheduler Execution

Launch the operating system in QEMU:
```bash
bash scripts/run_qemu.sh
```

**What Happens During Boot:**
1. **Limine Bootloader Handover:** Limine loads the kernel into the higher-half virtual address `0xffffffff80000000` and jumps to `_start`.
2. **CPU Control Register Configuration:** The kernel enables SSE/FPU in `CR0` and `CR4` to ensure safe stack zeroing and formatting.
3. **Descriptor Tables & Timer:** Loads the 64-bit flat GDT (`lgdt`), remaps the 8259 PIC vectors to `0x20-0x2F`, loads the IDT, and configures the 8254 PIT to 100 Hz.
4. **Memory Allocator:** Initializes the 128 KiB heap bump allocator and performs a 64-byte alignment test.
5. **Synchronization:** Acquires and releases `SpinLock<T>` with interrupt masking.
6. **Task Execution:** Spawns 4 heterogeneous tasks:
   - `Task-1-CPUBound` (Priority 5, Burst 18)
   - `Task-2-Interactive` (Priority 5, Burst 10, yields periodically)
   - `Task-3-SyncWorker` (Priority 8, Burst 14)
   - `Task-4-Diagnostics` (Priority 3, Burst 6)
7. **Dynamic Quantum Round Robin Loop:**
   - Under load $N=4$, dynamic quantum adapts to $9$ ticks. Task 1 is preempted at tick 10.
   - Task 2 cooperatively yields at tick 12.
   - Task 3 terminates at tick 25; under load $N=3$, quantum adapts to $7$ ticks.
   - Task 4 terminates at tick 31; under load $N=2$, quantum expands to $16$ ticks.
   - All tasks complete successfully, and CPU halts cleanly via `isa-debug-exit` port `0xf4`.

**Expected Live Serial Output:**
```text
================================================================
     __  __           _                             _   _       
     \ \/ /__ _  ___ | |____      ____ _ _ __  _   | |_| |__    
      \  // _` |/ __|| '_ \ \ /\ / / _` | '_ \| |_ | __| '_ \   
      / /| (_| |\__ \| | | \ V  V / (_| | | | |  _|| |_| | | |  
     /_/  \__,_||___/|_| |_|\_/\_/ \__,_|_| |_|_|   \__|_| |_|  
                                                                
                YASHWANTH MICRO-OS (x86_64)                    
        Educational Bare-Metal Kernel with Dynamic Quantum      
================================================================

[TICKS:000000] [INFO ] Bootloader handover complete. Running in 64-bit Long Mode.
[TICKS:000000] [INFO ] Project Variant: Minimal Round Robin Micro-OS with Dynamic Quantum
[TICKS:000000] [INFO ] Initializing Global Descriptor Table (GDT)...
[TICKS:000000] [INFO ] Initializing Interrupt Descriptor Table (IDT) & 8259 PIC...
[TICKS:000000] [INFO ] Configuring Intel 8254 PIT to 100 Hz...
[TICKS:000000] [MEM  ] Initializing Kernel Bump Allocator (128 KiB heap)...
[TICKS:000000] [MEM  ] Allocated 64 bytes at raw address 0xffffffff80017b00. Free memory: 131008 bytes
[TICKS:000000] [SYNC ] Testing kernel SpinLock<T> synchronization primitive...
[TICKS:000000] [SYNC ] Acquired spinlock. Incremented counter to 100
[TICKS:000000] [SYNC ] Re-acquired spinlock. Verified protected counter = 100

----------------------------------------------------------------
[TICKS:000000] [SCHED] Spawning controlled tasks for Dynamic Quantum Round Robin test:
----------------------------------------------------------------
[TICKS:000000] [TASK ] Spawned Task 1: 'Task-1-CPUBound'    (Burst: 18 ticks, Priority: 5)
[TICKS:000000] [TASK ] Spawned Task 2: 'Task-2-Interactive' (Burst: 10 ticks, Priority: 5)
[TICKS:000000] [TASK ] Spawned Task 3: 'Task-3-SyncWorker'  (Burst: 14 ticks, Priority: 8)
[TICKS:000000] [TASK ] Spawned Task 4: 'Task-4-Diagnostics' (Burst:  6 ticks, Priority: 3)

----------------------------------------------------------------
[TICKS:000000] [SCHED] Starting Integrated Scheduling Loop (PIT Driven Ticks):
----------------------------------------------------------------
[TICKS:000001] [SCHED] SWITCH: [INIT] -> [Task-1-CPUBound] | Reason: InitialSchedule | Dynamic Quantum: 9 ticks (Queue: 4)
[TICKS:000010] [SCHED] SWITCH: [Task-1-CPUBound] -> [Task-2-Interactive] | Reason: QuantumExpired (Preempt) | Dynamic Quantum: 9 ticks (Queue: 4)
[TICKS:000012] [TASK ] Task 2 voluntarily yields CPU to simulate I/O burst completion
[TICKS:000025] [SCHED] SWITCH: [Task-3-SyncWorker] -> [Task-4-Diagnostics] | Reason: TaskTerminated (Finished) | Dynamic Quantum: 7 ticks (Queue: 3)
[TICKS:000031] [SCHED] SWITCH: [Task-4-Diagnostics] -> [Task-1-CPUBound] | Reason: TaskTerminated (Finished) | Dynamic Quantum: 16 ticks (Queue: 2)
[TICKS:000040] [SCHED] SWITCH: [Task-1-CPUBound] -> [Task-2-Interactive] | Reason: TaskTerminated (Finished) | Dynamic Quantum: 12 ticks (Queue: 1)
[TICKS:000042] [TASK ] Task 2 voluntarily yields CPU to simulate I/O burst completion
[TICKS:000045] [TASK ] Task 2 voluntarily yields CPU to simulate I/O burst completion
[TICKS:000048] [TASK ] Task 2 voluntarily yields CPU to simulate I/O burst completion
[TICKS:000049] [SCHED] All tasks terminated. CPU entered idle state.

================================================================
                 SCHEDULER PERFORMANCE REPORT                   
================================================================
TID    NAME                 PRIO   RUNTIME  WAITING  TURNAROUND CSW    PREEMPT
----------------------------------------------------------------
1      Task-1-CPUBound      5      18       22       40         2      1     
2      Task-2-Interactive   5      10       39       49         5      0     
3      Task-3-SyncWorker    8      14       11       25         1      0     
4      Task-4-Diagnostics   3      6        25       31         1      0     
----------------------------------------------------------------
Completed Tasks:           4
Total Context Switches:    9
Total Scheduling Ticks:    49
Average Waiting Time:      24 ticks
Average Turnaround Time:   36 ticks
Scheduling Efficiency:     100% (No starvation observed)

================================================================
           YASHWANTH MICRO-OS EXECUTION SUCCESSFUL             
================================================================
```

---

## 7. Step 6: Automated Trace & Fairness Analysis

Parse the serial log captured by QEMU:
```bash
python3 scripts/analyze_traces.py traces/qemu_serial.log
```

**Expected Output:**
```text
=================================================================
      YASHWANTH MICRO-OS - PYTHON TRACE ANALYSIS TOOL
=================================================================
Target Trace Log: traces/qemu_serial.log

Total Monotonic Ticks:    49 (approx 490 ms)
Total Context Switches:   5
Switch Reason Breakdown:
  - InitialSchedule       : 1
  - QuantumExpired        : 1
  - TaskTerminated        : 3

Memory Allocations Logged:
  - Total Allocation Count: 1
  - Total Bytes Allocated:  64 bytes

Jain's Fairness Index:    0.8780 (1.0000 = perfectly fair)
=================================================================

[OK] Performance metrics exported to results/performance_metrics.json
```

---

## 8. Step 7: Interactive Hugging Face Simulator Execution

To run the companion simulator locally:
```bash
python3 hf_space/app.py
```
Open `http://localhost:7860` in a web browser. Adjust task burst times and observe the interactive Gantt chart comparing Static Round Robin with Dynamic Quantum Round Robin.
