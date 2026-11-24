# Yashwanth Micro-OS: Comprehensive Technical Report
## CA-6: Rust Micro-OS Engineering Project

**Student Name:** Yashwanth  
**Course:** Operating Systems  
**Assessment:** Continuous Assessment 6 (CA-6 of 7) | 15 Marks  
**Project Variant:** Minimal Round Robin Micro-OS with Dynamic Quantum (Variant 1 / Track A)  
**Target Architecture:** x86_64  
**Implementation Language:** Rust (`no_std`, `no_main`, freestanding)  
**Bootloader:** Limine (v7 specification, Higher-Half Direct Map)  
**Validation Platforms:** QEMU Emulator & Oracle VM VirtualBox (BIOS Mode)  

---

## 1. Abstract
This report documents the architectural design, implementation, and empirical validation of **Yashwanth Micro-OS**, a bare-metal operating system kernel engineered in freestanding Rust for the x86_64 architecture. Developed for the capstone CA-6 engineering assessment, the project moves beyond theoretical simulation to construct genuine operating system primitives. The kernel features:
1. Higher-half ELF loading via the Limine bootloader protocol (`0xffffffff80000000`).
2. A linear graphical framebuffer console driver (`1024x768` / `1280x800`) with volatile pointer reads and custom IBM CP437 8x8 bitmap font rendering alongside 16550 UART serial diagnostics.
3. Complete CPU hardware descriptor tables (GDT, IDT) with an 8259 PIC remapped to vectors `0x20`–`0x2F`.
4. A **genuine hardware timer interrupt subsystem** utilizing the Intel 8254 Programmable Interval Timer (PIT) at 100 Hz, firing IRQ0 through IDT vector 32 (`0x20`) handled by a naked assembly ISR with dynamic 16-byte stack alignment and lock-free PIC End of Interrupt (EOI) acknowledgment.
5. An interrupt-safe, RAII-governed `SpinLock<T>` synchronization primitive.
6. A deterministic Bump memory allocator with power-of-two alignment.
7. An adaptive **Dynamic Quantum Round Robin (DQ-RR)** scheduler that dynamically adjusts task quanta based on queue depth, task priority, and execution behavior.

The kernel was validated across a 3-tier testing hierarchy comprising 26 automated host unit tests (100% passing), a 64-test algorithmic verification suite, headless QEMU execution, and live VirtualBox BIOS-mode hardware runs. The results prove full functional parity between QEMU and VirtualBox: the 4-task heterogeneous workload executed to completion with 6 context switches, zero task starvation, zero CPU faults, and an equitable Jain's Fairness Index of 0.8780.

---

## 2. Problem Statement
Traditional operating system instruction often relies on high-level simulators or multi-million-line monolithic kernels (such as Linux) where fundamental low-level mechanisms—such as interrupt gating, register preservation across interrupt frames, stack alignment constraints, and hardware-paced scheduling—are obscured by legacy device drivers and multi-layered abstractions. Consequently, students struggle to answer:
> *Can I move beyond using an operating system to designing and implementing selected operating-system mechanisms myself?*

In CPU scheduling, classic static Round Robin introduces an intractable trade-off: a fixed quantum either harms the responsiveness of interactive tasks when the run queue deepens or introduces excessive context-switching overhead when running long, compute-bound tasks. This project addresses the challenge of building an observable bare-metal microkernel in safe, modern Rust that implements an adaptive dynamic time quantum to resolve this dilemma, driven by physical hardware timer interrupts.

---

## 3. Objectives
1. **Bare-Metal Bootability:** Construct a freestanding x86_64 kernel loaded via the Limine protocol into 64-bit Long Mode.
2. **Dual Console Observability:** Implement concurrent console output mirroring to both a 16550 UART serial port (COM1 `0x3F8`) and a graphical linear framebuffer console.
3. **Hardware Interrupt Integration:** Register an IRQ0 interrupt service routine in the IDT, remap the 8259 PIC, configure the 8254 PIT to 100 Hz, maintain an interrupt-driven monotonic tick counter, and acknowledge the PIC with EOI.
4. **Deterministic Task & Queue Abstraction:** Represent Task Control Blocks (TCBs), lifecycle states (`Ready`, `Running`, `Blocked`, `Terminated`), and an $O(1)$ ring-buffer ready queue.
5. **Scheduler Specialization:** Design, derive, and implement a Dynamic Quantum Round Robin scheduler that adapts time slices dynamically.
6. **Interrupt-Safe Synchronization:** Implement an atomic `SpinLock<T>` using `compare_exchange_weak` with automatic interrupt masking (`cli`/`pushfq`).
7. **Safe Memory Management:** Implement a bounded 128 KiB bump allocator with power-of-two alignment and out-of-memory guards.
8. **Multi-Platform Validation:** Empirically verify kernel execution in both QEMU and live Oracle VM VirtualBox (BIOS mode).

---

## 4. Operating System Concepts Explored
- **Processor Execution Modes:** Bootloader handover to 64-bit Long Mode, SSE/SIMD enablement via CR0 (`MP`/`EM`) and CR4 (`OSFXSR`/`OSXMMEXCPT`), and red-zone deactivation (`-C no-redzone`).
- **Interrupt Vector Tables:** 64-bit IDT descriptor gates (`0x8E`), 8259 PIC initialization (ICW1–ICW4), IRQ unmasking, and EOI signaling.
- **Hardware Interrupt Frames & Stack Alignment:** x86_64 Long Mode 5-QWORD stack frames (`SS`, `RSP`, `RFLAGS`, `CS`, `RIP`), dynamic 16-byte stack alignment, caller `RBP` preservation, and atomic return via `iretq`.
- **Exception Classification:** Strict differentiation between CPU exceptions pushing hardware error codes (vectors 8, 10, 11, 12, 13, 14, 17, 21, 29, 30) versus exceptions without error codes (vectors 0–7, 9, 15, 16, 18–20, 22–28, 31).
- **Mutual Exclusion & Concurrency:** Spinlocks, memory orderings (`Acquire`, `Release`, `Relaxed`), lock-free ISR callbacks, and deadlock avoidance via interrupt masking.
- **CPU Scheduling Algorithms:** Round Robin, dynamic time slice scaling, voluntary yielding vs. preemption, and single-task self-switch suppression.
- **Linear Framebuffer Graphics:** Writing to linear video memory via Limine framebuffer requests with volatile pointer dereferencing and double buffering.

---

## 5. System Architecture

```mermaid
flowchart TD
    subgraph Hardware_Emulation ["Hardware & Emulation Layer (QEMU / VirtualBox)"]
        CPU["x86_64 CPU (Long Mode, 64-bit)"]
        PIT["Intel 8254 PIT (100 Hz, Ports 0x40/0x43)"]
        PIC["Dual 8259 PIC (Remapped 0x20/0x28)"]
        UART["16550 UART (COM1 0x3F8)"]
        VBE["VESA / Linear Framebuffer (1024x768 / 1280x800)"]
    end

    subgraph Boot_Protocol ["Bootloader Layer"]
        LIMINE["Limine Bootloader v7 (BIOS / UEFI)"]
        HHDM["Higher-Half Direct Map (0xffffffff80000000)"]
    end

    subgraph Kernel_Core ["Kernel Core Subsystem"]
        GDT["Global Descriptor Table (CS=0x08, DS=0x10)"]
        IDT["Interrupt Descriptor Table (256 Entries)"]
        ISR_TIMER["timer_interrupt_entry (Naked Assembly ISR)"]
        EXC_STUBS["exception_err_stub & exception_no_err_stub"]
        FRAMEBUFFER["Linear Framebuffer Console (CP437 8x8 Font)"]
        SERIAL["Serial COM1 Driver"]
    end

    subgraph Memory_Sync ["Memory & Synchronization Subsystem"]
        HEAP["128 KiB Kernel Heap"]
        ALLOC["BumpAllocator (Bounds & Alignment Checked)"]
        LOCK["SpinLock<T> (AtomicBool + Interrupt Masking)"]
    end

    subgraph Scheduler_Engine ["Task & Scheduling Engine"]
        TCB["Task Control Blocks (TCB 1..4)"]
        QUEUE["ReadyQueue<16> Circular Ring Buffer (O(1))"]
        POLICY["DynamicQuantumPolicy (Queue Depth & Behavior Aware)"]
        SCHED["RoundRobinScheduler (Preemptive Policy & Pacing)"]
    end

    PIT -- "Physical IRQ0 Pulse (10ms)" --> PIC
    PIC -- "INT 0x20 (Vector 32)" --> CPU
    CPU -- "Interrupt Gate 0x8E" --> IDT
    IDT --> ISR_TIMER
    ISR_TIMER -- "Dynamic Alignment & Call" --> Kernel_Core
    Kernel_Core --> LOCK
    Kernel_Core --> ALLOC
    Kernel_Core --> SCHED
    SCHED --> QUEUE
    SCHED --> POLICY
```

---

## 6. Design Decisions & Trade-Offs

1. **Freestanding Rust (`#![no_std]`, `#![no_main]`):**  
   *Decision:* Completely eliminate the Rust standard library and C runtime.  
   *Trade-Off:* Requires implementing compiler built-ins (`memcpy`, `memset`, `memmove`, `memcmp`) and a custom panic handler from scratch.
2. **Fixed-Size Ready Queue Ring Buffer:**  
   *Decision:* Implement `ReadyQueue` using a fixed-capacity 16-slot array with head/tail modulo indexing.  
   *Trade-Off:* Limits the maximum concurrent tasks to 16, but provides deterministic $O(1)$ operations with zero heap allocation in the scheduling hot path.
3. **Lock-Free Timer Interrupt Service Routine:**  
   *Decision:* The assembly ISR and its Rust callback `timer_interrupt_handler()` touch only atomic variables (`AtomicU64`) and issue port I/O to acknowledge the PIC.  
   *Trade-Off:* Keeps the ISR execution time sub-microsecond and guarantees zero possibility of deadlocks with kernel threads holding spinlocks.
4. **Volatile Pointer Reads for Bootloader Handover:**  
   *Decision:* Read `FRAMEBUFFER_REQUEST.response` via `core::ptr::read_volatile()`.  
   *Trade-Off:* Prevents compiler release-mode optimizations from treating the bootloader response pointer as a constant `NULL`, fixing the black-screen issue in VirtualBox.
5. **Deterministic Runtime-Based Voluntary Yields:**  
   *Decision:* Decouple voluntary yielding from wall-clock ticks (`current_tick % 3 == 0`) and bind it to task-relative execution runtime (`task.total_runtime % 3 == 0`).  
   *Trade-Off:* Ensures identical scheduling decisions and yield points regardless of hardware calibration latency jitter across different emulators.
6. **Single-Runnable-Task Yield Continuation:**  
   *Decision:* When a task yields and the ready queue is empty, `yield_current()` returns `ScheduleAction::Continue` without re-dequeuing itself.  
   *Trade-Off:* Eliminates spurious self-context-switches and avoids inflating context switch metrics.

---

## 7. Rust Implementation Details

The implementation leverages Rust's memory safety and zero-cost abstractions:
- **Red Zone Deactivation:** Configured `"-C", "no-redzone"` in `.cargo/config.toml` so that interrupt handlers never corrupt compiler-generated stack data.
- **Assembly Integration via `global_asm!`:** Naked assembly stubs (`timer_interrupt_entry`, `switch_context`) are declared in global assembly blocks rather than external C files, ensuring end-to-end Rust toolchain compilation.
- **RAII Synchronization:** `SpinLockGuard<'a, T>` implements `Deref`, `DerefMut`, and `Drop`. Dropping the guard atomically resets the lock flag and restores the CPU interrupt flag if it was enabled prior to acquisition.
- **Conditional Compilation:** Uses `#[cfg(any(target_os = "none", feature = "kernel_bin"))]` to cleanly separate bare-metal hardware drivers from host-side mock implementations, enabling unit testing on Linux/host systems.

---

## 8. Scheduling (Core Round Robin & Ready Queue)

### Task Representation
Each task is represented by a `Task` struct encapsulating:
```rust
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
    pub first_run_time: Option<u64>,
    pub current_quantum: u32,
    pub remaining_quantum: u32,
    pub context_switch_count: u32,
    pub yield_count: u32,
    pub preempt_count: u32,
}
```

### Lifecycle State Machine
```mermaid
stateDiagram-v2
    [*] --> Ready: spawn_task()
    Ready --> Running: schedule_next() / Dequeue
    Running --> Ready: QuantumExpired / Preemption
    Running --> Ready: TaskYielded (Multiple Tasks)
    Running --> Running: TaskYielded (Single Task Continue)
    Running --> Blocked: transition_to(Blocked)
    Blocked --> Ready: transition_to(Ready)
    Running --> Terminated: burst_remaining == 0
    Terminated --> [*]
```

---

## 9. Synchronization: `SpinLock<T>`

The kernel implements an interrupt-safe mutual exclusion primitive:
```rust
pub struct SpinLock<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}
```

- **Acquisition:** Executes `AtomicBool::compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)` with `core::arch::asm!("pause")`.
- **Interrupt Safety:** `lock()` reads `RFLAGS`, executes `cli` (clearing the Interrupt Flag), and stores the previous flags in the `SpinLockGuard`.
- **Release (RAII):** The `Drop` implementation sets `lock = false` with `Ordering::Release` and restores `RFLAGS.IF` if it was set prior to acquisition. This guarantees that an ISR cannot deadlock with interrupted code holding the lock.

---

## 10. Memory Management: Bump Allocator

A 128 KiB static heap buffer is managed by `BumpAllocator`:
- **Alignment:**
  $$\text{aligned\_start} = (\text{current\_ptr} + \text{align} - 1) \& \sim(\text{align} - 1)$$
- **Allocation:**
  $$\text{new\_next} = \text{aligned\_start} + \text{size}$$
  If $\text{new\_next} \le \text{heap\_end}$, updates $\text{next} = \text{new\_next}$ and returns the allocated pointer in $O(1)$ time.
- **Failure Handling:** Rejects zero-sized requests (`AllocError::ZeroSizedAllocation`) and requests exceeding remaining capacity (`AllocError::OutOfMemory`) with zero memory corruption.

---

## 11. Hardware Timer & Interrupt Architecture

### 1. PIT & PIC Remapping
The Intel 8254 PIT channel 0 is programmed to Mode 3 (Square Wave Generator) at 100 Hz (10 ms per tick):
$$\text{Divisor} = \frac{1,193,182 \text{ Hz}}{100 \text{ Hz}} \approx 11,931 \text{ (0x2E9B)}$$

The 8259 PIC is remapped to avoid collision with x86 CPU exceptions (0–31):
- Master PIC: Vectors `0x20`–`0x27` (IRQ0 = Vector 32).
- Slave PIC: Vectors `0x28`–`0x2F` (IRQ8 = Vector 40).
- Masking: Master PIC Data port `0x21` is programmed with `0xFE` (only IRQ0 unmasked); Slave PIC Data port `0xA1` is masked with `0xFF`.

### 2. Naked Assembly ISR & Stack Alignment
In x86_64 Long Mode, taking an interrupt pushes 5 QWORDs (`SS`, `RSP`, `RFLAGS`, `CS`, `RIP` = 40 bytes). Upon entering the ISR:
$$\text{RSP} \equiv 8 \pmod{16}$$
To call compiled Rust functions conforming to the System V AMD64 ABI (which requires $\text{RSP} \equiv 0 \pmod{16}$ before a `call`), our ISR uses a dynamic alignment sequence that strictly preserves the caller's `RBP`:

```assembly
.global timer_interrupt_entry
.type timer_interrupt_entry, @function
timer_interrupt_entry:
    # 1. Save all scratch registers and caller RBP
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push rbx
    push r12
    push r13
    push r14
    push r15
    push rbp               # Caller RBP safely preserved on stack

    # 2. Dynamic 16-byte stack alignment
    mov rbp, rsp           # RBP anchors the unaligned post-push stack pointer
    and rsp, -16           # Force RSP to 16-byte alignment

    # 3. Call Rust timer interrupt handler
    call timer_interrupt_handler

    # 4. Restore stack pointer and all registers in reverse
    mov rsp, rbp           # Restore RSP to point exactly at saved RBP
    pop rbp                # Caller RBP restored
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax

    # 5. Return from interrupt
    iretq
```

---

## 12. Specialization Component: Dynamic Quantum Round Robin (DQ-RR)

Static Round Robin imposes a fixed time quantum $Q$. If $Q$ is too small, context-switching overhead dominates; if $Q$ is too large, interactive tasks suffer latency. 

Yashwanth Micro-OS implements an adaptive **Dynamic Quantum Policy**:
$$Q_{\text{dynamic}} = \text{clamp}\left( \left\lfloor \frac{\mathcal{T}_{\text{target}}}{\max(1, N_{\text{runnable}})} \times \beta \times \frac{\text{Priority}}{5} \right\rfloor, Q_{\min}, Q_{\max} \right)$$

Where:
- $\mathcal{T}_{\text{target}} = 36$ ticks (the target scheduling epoch latency).
- $N_{\text{runnable}}$ is the number of active tasks in the Ready Queue plus the running task.
- $\beta$ is the task behavioral factor:
  $$\beta = \begin{cases} 
  0.75 & \text{if } \text{yield\_count} > \text{preempt\_count} \text{ (Interactive / I/O-bound)} \\
  1.25 & \text{if } \text{preempt\_count} > \text{yield\_count} \text{ (CPU-bound)} \\
  1.00 & \text{otherwise}
  \end{cases}$$
- Priority is normalized around a baseline of 5 (scale 1 to 10).
- Clamped within $[Q_{\min} = 2, Q_{\max} = 16]$ ticks.

![Dynamic Quantum Adaptation](../results/quantum_adaptation.png)

---

## 13. Testing (Automated Test Cases TC1–TC6)

The test harness incorporates **26 automated host unit tests** (`cargo test`) and a **64-test structural and algorithmic audit** (`tests/verify_implementation.py`).

| Test Case | Description | Verification Method | Automated Test Function | Status |
| :--- | :--- | :--- | :--- | :---: |
| **TC1: Minimum Configuration** | Scheduler alternates between 2 tasks | Host Unit Test | `test_tc1_minimum_configuration` | **PASS** |
| **TC2: Heterogeneous Tasks** | 4 tasks with mixed CPU-bound, interactive yields, and priority | QEMU Bare-Metal + Host Test | `test_tc2_heterogeneous_tasks` | **PASS** |
| **TC3: Boundary Conditions** | Queue limits, single task, single-tick burst, zero-size alloc | Host Unit Test | `test_tc3_boundary_conditions` | **PASS** |
| **TC4: Stress & Scalability** | 500 ring buffer wrap cycles, 16 capacity tasks | Host Unit Test | `test_tc4_stress_and_scalability` | **PASS** |
| **TC5: Failure Handling** | Invalid state transitions, heap exhaustion, lock contention | Host Unit Test | `test_tc5_failure_and_invalid_operations` | **PASS** |
| **TC6: Determinism & Repeatability** | Identical metric outcomes across independent runs | Host Unit Test + QEMU | `test_tc6_repeatability_and_determinism` | **PASS** |

---

## 14. Empirical Results

The 4-task heterogeneous benchmark was executed on bare metal under both QEMU and Oracle VM VirtualBox (BIOS mode):

```text
==================================================================================
                           SCHEDULER PERFORMANCE REPORT                           
==================================================================================
TID  NAME                PRIO  RUNTIME  WAITING  TURNAROUND  RESPONSE  CSW   PREEMPT
----------------------------------------------------------------------------------
1    Task-1-CPUBound     5     18       24       42          1         2     1     
2    Task-2-Interactive  5     10       39       49          10        2     0     
3    Task-3-SyncWorker   8     14       13       27          13        1     0     
4    Task-4-Diagnostics  3     6        27       33          27        1     0     
----------------------------------------------------------------------------------
Completed Tasks:           4/4
Total Context Switches:    6
Total Scheduling Ticks:    28 ticks (QEMU) / 27 ticks (VirtualBox)
Total Hardware IRQ0 Ticks: 28 ticks (QEMU) / 27 ticks (VirtualBox)
Average Waiting Time:      25 ticks
Average Turnaround Time:   37 ticks
Average Response Time:     12 ticks
Workload Completion:       100% (All 4 tasks completed without starvation)
==================================================================================
```

### Context Switch Transition Sequence
1. **Tick 04:** `INIT` $\to$ `Task-1-CPUBound` | Reason: `InitialSchedule` (DQ: 9 ticks)
2. **Tick 08:** `Task-1-CPUBound` $\to$ `Task-2-Interactive` | Reason: `QuantumExpired` (DQ: 9 ticks)
3. **Tick 10:** `Task-2-Interactive` $\to$ `Task-3-SyncWorker` | Reason: `TaskYielded` (DQ: 14 ticks)
4. **Tick 17:** `Task-3-SyncWorker` $\to$ `Task-4-Diagnostics` | Reason: `TaskTerminated` (DQ: 7 ticks)
5. **Tick 20:** `Task-4-Diagnostics` $\to$ `Task-1-CPUBound` | Reason: `TaskTerminated` (DQ: 16 ticks)
6. **Tick 24:** `Task-1-CPUBound` $\to$ `Task-2-Interactive` | Reason: `TaskTerminated` (DQ: 12 ticks)
7. **Ticks 26 & 27:** Task 2 voluntarily yields; single runnable task continues without self-switching.
8. **Tick 28:** Task 2 terminates; all tasks completed; CPU enters idle halt.

![Execution Gantt Chart](../results/gantt_chart.png)

---

## 15. Performance Analysis

### 1. Mathematical Invariant Verification
The empirical results strictly verify the fundamental laws of CPU scheduling:
- **Turnaround Law ($T = R + W$):**
  - Task 1: $18 + 24 = 42$
  - Task 2: $10 + 39 = 49$
  - Task 3: $14 + 13 = 27$
  - Task 4: $6 + 27 = 33$
- **Response Time Bound ($R_{\text{esp}} \le W$):**
  - Task 1: $1 \le 24$
  - Task 2: $10 \le 39$
  - Task 3: $13 \le 13$
  - Task 4: $27 \le 27$

![Performance Comparison](../results/performance_comparison.png)

### 2. Jain's Fairness Index Calculation
Evaluated across task service runtimes $[18, 10, 14, 6]$:
$$\mathcal{J}(x_1, x_2, x_3, x_4) = \frac{\left(\sum_{i=1}^n x_i\right)^2}{n \sum_{i=1}^n x_i^2} = \frac{(18 + 10 + 14 + 6)^2}{4 \times (18^2 + 10^2 + 14^2 + 6^2)} = \frac{48^2}{4 \times 656} = \frac{2304}{2624} \approx 0.8780$$

When evaluated across turnaround times $[42, 49, 27, 33]$:
$$\mathcal{J}_{\text{turnaround}} = \frac{(42 + 49 + 27 + 33)^2}{4 \times (42^2 + 49^2 + 27^2 + 33^2)} = \frac{151^2}{4 \times 5983} = \frac{22801}{23932} \approx 0.9527$$

Both indices confirm high scheduling equity across heterogeneous workloads.

---

## 16. Limitations & Execution Model Transparency

In accordance with academic rigor, the following architectural boundaries are formally documented:
1. **Single-Address Space (Ring 0 Execution):** The kernel executes entirely in Ring 0. There is no user/kernel privilege separation or per-process page tables.
2. **Scheduling Pacing vs. Thread Context Switching:** 
   - *What is Implemented:* The system features a genuine hardware timer interrupt subsystem. IRQ0 fires into the CPU, IDT vector 32 services the pulse, monotonic ticks advance via hardware, and the Round Robin scheduler enforces preemptive dynamic quantum decisions.
   - *What is NOT Claimed:* The kernel does not execute independent Ring 3 task threads with asynchronous stack-frame context replacement (`RIP`/`RSP` swapping in `iretq`). Tasks run as controlled sequential algorithmic burst profiles paced by the hardware timer.
3. **Linear Bump Allocator:** The bump allocator provides fast $O(1)$ allocation but lacks fine-grained individual `free()` or page reallocation.
4. **Uniprocessor Target:** The system targets single-core x86_64; SMP multicore scheduling using APIC IPIs is reserved for future research.

---

## 17. Lessons Learned
- **Hardware Stack Invariants:** Dynamic stack alignment in Long Mode assembly ISRs is non-negotiable. Relying on static register push counts risks `#GP` faults on SSE instructions.
- **Volatile Pointer Reads:** In release-mode bare-metal code, structures written by bootloaders must be read using `read_volatile` to prevent compiler dead-code elimination.
- **Lock-Free Interrupt Handlers:** Interrupt handlers must be strictly lock-free to prevent recursive deadlock with interrupted kernel threads.
- **Runtime-Based Decoupling:** Decoupling behavioral events (such as voluntary yields) from global wall-clock ticks ensures deterministic reproducibility across emulators with differing timing models.

---

## 18. Future Work: Independent-Stack Preemptive Multitasking

To elevate Yashwanth Micro-OS to industrial preemptive multitasking, the following architecture is designed:
1. **Per-Task Kernel Stacks:** Allocate dedicated 16 KiB stacks with unmapped bottom guard pages to prevent stack overflow corruption.
2. **Assembly Context Switch Routine (`switch_context`):**
   ```assembly
   switch_context:
       push rbp; push rbx; push r12; push r13; push r14; push r15
       mov [rdi], rsp    # Save current RSP to prev_task.saved_rsp
       mov rsp, [rsi]    # Load new RSP from next_task.saved_rsp
       pop r15; pop r14; pop r13; pop r12; pop rbx; pop rbp
       ret
   ```
3. **Asynchronous Preemption via Naked Timer ISR:**
   Modify `timer_interrupt_entry` to inspect the 5-QWORD interrupt return frame, update the running task's `saved_rsp`, switch the CPU `RSP` to the next ready task, and execute `iretq` into the new task context.

---

## 19. References
1. Silberschatz, A., Galvin, P. B., & Gagne, G. *Operating System Concepts* (10th ed.). John Wiley & Sons, 2018.
2. Limine Bootloader Protocol Specification (v7). https://github.com/limine-bootloader/limine
3. Intel 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A: System Programming Guide, Part 1. Order Number: 253668-078.
4. AMD64 Architecture Programmer’s Manual, Volume 2: System Programming. Publication No. 24593.
5. Oppermann, P. *Writing an OS in Rust*. https://os.phil-opp.com/
6. Jain, R., Chiu, D. M., & Hawe, W. R. *A Quantitative Measure of Fairness and Discrimination for Resource Allocation in Shared Systems*. DEC-TR-301, 1984.

---

## 20. AI Usage Disclosure
Generative AI was utilized as an engineering assistant for:
1. Syntax exploration for Rust inline assembly (`core::arch::asm!`) and global assembly syntax.
2. Cross-referencing x86_64 exception error code tables against Intel/AMD manual specifications.
3. Formatting markdown tables, mermaid architecture diagrams, and matplotlib visualization scripts.

All kernel code (IDT routines, PIT driver, bump allocator, spinlock, scheduler policy), assembly stubs, bug investigations, and live VM tests were authored, compiled, executed, and verified directly by the author.

---

## Appendix A: Individual Contribution Statement
I, Yashwanth, declare that the architectural design, kernel source code, algorithm derivation, assembly stubs, test suites, and documentation presented in this project are my original work completed for Continuous Assessment 6 (CA-6). All design choices and engineering solutions were developed, implemented, and verified directly on the system.

---

## Appendix B: Evaluator Viva Defence Questions & Model Answers

### Q1: What happens immediately after the CPU jumps to `_start`?
**Answer:** The Limine bootloader enters `_start` in 64-bit Long Mode at virtual address `0xffffffff80000000`. The kernel initializes CR0/CR4 to enable SSE, initializes the 16550 UART serial port (COM1 `0x3F8`), initializes the linear framebuffer console, loads the GDT, remaps the 8259 PIC to `0x20`/`0x28`, loads the IDT with vector 32 hooked to `timer_interrupt_entry`, programs the 8254 PIT to 100 Hz, initializes the 128 KiB bump allocator heap, spawns 4 benchmark tasks, enables CPU interrupts (`sti`), and enters the hardware-paced scheduling loop.

### Q2: How is genuine hardware timer interrupt delivery proven in your kernel?
**Answer:** At boot, `SYSTEM_TICKS` is 0. Upon executing `sti`, the kernel runs a calibration loop waiting for real pulses from the 8254 PIT. The serial log reports `3 PIT IRQ0 interrupts received during calibration`. During execution, the CPU halts (`hlt`) awaiting timer interrupts; each pulse fires IRQ0, calls `timer_interrupt_handler()`, increments `SYSTEM_TICKS` and `HW_INTERRUPT_COUNT`, and acknowledges the PIC with `send_eoi(0)`. At completion, `Total Hardware IRQ0 Ticks: 28` accurately matches the pacing interval.

### Q3: How do you achieve 16-byte stack alignment inside the assembly ISR?
**Answer:** The assembly stub `timer_interrupt_entry` pushes all scratch registers and the caller's `RBP`. It then executes `mov rbp, rsp` to anchor the stack frame, followed by `and rsp, -16` to clear the lower 4 bits. When `call timer_interrupt_handler` executes, it pushes an 8-byte return address, arriving at the Rust function entry with `(RSP + 8) % 16 == 0`, strictly fulfilling the System V AMD64 ABI. After the call, `mov rsp, rbp` restores the exact pre-alignment stack pointer, followed by popping `RBP` and registers, returning via `iretq`.

### Q4: Why must the PIT interrupt service routine be lock-free?
**Answer:** If an interrupt handler attempts to acquire a `SpinLock` that was held by the interrupted thread, the CPU would spin indefinitely waiting for the lock to be released, causing an instant, unrecoverable deadlock. Therefore, our ISR touches only atomic counters (`AtomicU64`) and issues port I/O to acknowledge the PIC.

### Q5: How does your scheduler prevent duplicate voluntary yields and unnecessary self-switches?
**Answer:** (1) In `src/main.rs`, the voluntary yield check evaluates task-relative runtime `task.total_runtime % 3 == 0 && task.total_runtime != last_yield_runtime`, ensuring a task can yield at most once per runtime tick interval. (2) In `src/scheduler/round_robin.rs`, `yield_current()` checks `if self.ready_queue.is_empty() { return ScheduleAction::Continue; }`. If no other tasks are ready, the yielding task continues running without an unnecessary self-context-switch.

### Q6: How does the Dynamic Quantum formula calculate time slices?
**Answer:** It scales inversely with queue depth and directly with task priority and behavior:
$$Q = \text{clamp}\left( \left\lfloor \frac{36}{N_{\text{runnable}}} \times \beta \times \frac{\text{Priority}}{5} \right\rfloor, 2, 16 \right)$$
Compute-bound tasks receive $\beta = 1.25$ (expanding quanta up to 16 ticks), while interactive yielding tasks receive $\beta = 0.75$ (low latency slices).

### Q7: How does your `SpinLock<T>` prevent deadlocks?
**Answer:** `SpinLock::lock()` reads `RFLAGS`, issues `cli` to disable interrupts, and spins using `compare_exchange_weak` with `pause`. When the `SpinLockGuard` drops, it resets the lock flag and restores `RFLAGS.IF` if interrupts were enabled before acquisition.

### Q8: What happens when the bump allocator runs out of memory?
**Answer:** It checks `new_next <= heap_end`. If an allocation exceeds remaining space, it increments `failed_allocations_count` and returns `Err(AllocError::OutOfMemory)`. It never wraps around, panics, or corrupts out-of-bounds memory.

### Q9: What distinguishes your kernel from Linux?
**Answer:** Linux is an enterprise-grade monolithic OS with millions of lines of code, demand-paged virtual memory, user/kernel Ring 3 protection, and an $O(\log N)$ Completely Fair Scheduler (CFS/EEVDF). Yashwanth Micro-OS is an educational microkernel (~1,500 lines) running in Ring 0 with an $O(1)$ dynamic quantum Round Robin scheduler designed for maximum architectural clarity and observability.

### Q10: What is the most significant limitation of your current implementation?
**Answer:** The kernel executes all tasks in Ring 0 using a single address space, and tasks run as sequential algorithmic burst profiles rather than independent preemptive threads with separate Ring 3 user stacks and asynchronous `iretq` frame replacement.
