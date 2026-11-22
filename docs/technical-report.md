# Yashwanth Micro-OS: Comprehensive Technical Report
## CA-6: Rust Micro-OS Engineering Project

**Student Name:** Yashwanth  
**Course:** Operating Systems  
**Assessment:** Continuous Assessment 6 (CA-6 of 7) | 15 Marks  
**Project Variant:** Minimal Round Robin Micro-OS with Dynamic Quantum  
**Target Architecture:** x86_64  
**Implementation Language:** Rust (`no_std`, `no_main`)  
**Bootloader:** Limine  
**Emulation Platform:** QEMU  

---

## 1. Abstract
This report documents the design, implementation, and empirical validation of **Yashwanth Micro-OS**, a minimal educational operating system kernel developed in bare-metal Rust for the x86_64 architecture. Developed for the flagship CA-6 engineering assessment, the project moves beyond theoretical concepts to constructing working operating system primitives. The kernel features a Limine-compliant bootloader entry, 16550 UART serial diagnostics, an Interrupt Descriptor Table (IDT), an Intel 8254 Programmable Interval Timer (PIT), an interrupt-safe `SpinLock<T>` synchronization primitive, and a deterministic Bump memory allocator. The core specialization is **Scheduler Engineering**, featuring an adaptive **Dynamic Quantum Round Robin (DQ-RR)** scheduling engine that dynamically recalibrates execution time slices based on ready queue depth, task priority, and historical execution behavior (CPU-bound vs voluntary cooperative yields). The system was validated using a dual testing hierarchy comprising automated host-side unit tests and full bare-metal QEMU integration runs with Python serial trace analysis. Results demonstrate that DQ-RR achieves a 22.8% reduction in interactive turnaround latency and a 22.2% reduction in context-switching overhead compared to static Round Robin, while maintaining a Jain's Fairness Index of 0.9124.

---

## 2. Problem Statement
Traditional operating system instruction often relies either on purely theoretical models or on complex, multi-million-line production kernels (such as Linux) where fundamental mechanisms are obscured by architectural legacy and hardware drivers. Consequently, students struggle to answer:
> *Can I move beyond using an operating system to designing and implementing selected operating-system mechanisms myself?*

Specifically, in CPU scheduling, static Round Robin suffers from a fundamental trade-off: a fixed quantum either harms interactive response times under heavy loads or incurs excessive context-switching overhead under light, compute-bound workloads. This project addresses the engineering challenge of building a bare-metal, observable microkernel in safe, modern Rust that implements an adaptive dynamic time quantum to resolve this dilemma.

---

## 3. Objectives
1. **Bootable Kernel**: Construct a working x86_64 bare-metal kernel loaded via the modern Limine bootloader protocol in 64-bit Long Mode.
2. **Observability & Diagnostics**: Implement an unbuffered 16550 UART serial console and structured logging macros (`log_info!`, `log_sched!`, etc.) for real-time telemetry.
3. **Task & Queue Abstraction**: Represent task control blocks (TCBs), lifecycle states (`Ready`, `Running`, `Blocked`, `Terminated`), and an $O(1)$ ring-buffer ready queue.
4. **Scheduler Specialization**: Design, mathematically derive, and implement a Dynamic Quantum Round Robin scheduler.
5. **Hardware Timer Integration**: Configure the Intel 8254 PIT to generate 100 Hz periodic timer ticks for time slice accounting and preemption.
6. **Kernel Synchronization**: Implement an interrupt-safe, RAII-governed `SpinLock<T>` using atomic operations (`AtomicBool`).
7. **Memory Management**: Implement a bounded Bump (Linear) heap allocator with alignment enforcement and out-of-memory guards.
8. **Empirical Validation**: Validate the kernel using automated unit tests and QEMU execution traces parsed with a dedicated Python analysis pipeline.

---

## 4. Operating System Concepts Explored
- **Processor Execution Modes**: Transition from bootloader to 64-bit Long Mode flat address space.
- **Task Control Blocks (TCBs)**: State preservation, execution budgets, and CPU register contexts.
- **Context Switching**: Callee-saved register manipulation (`rsp`, `rbp`, `rbx`, `r12`..`r15`, `rflags`, `rip`).
- **CPU Scheduling Algorithms**: First-Come-First-Served (FCFS), Shortest Job First (SJF), Static Round Robin (RR), and Dynamic Quantum Round Robin (DQ-RR).
- **Hardware Interrupt Dispatching**: PIC 8259 vector remapping and Interrupt Descriptor Tables (IDT).
- **Mutual Exclusion & Concurrency**: Atomic memory operations (`compare_exchange_weak`), memory orderings (`Acquire` / `Release`), and interrupt-disabled critical sections.
- **Memory Allocation & Alignment**: Byte-level pointer arithmetic, memory bounds checking, and boundary alignment.

---

## 5. System Architecture
The system is divided into modular layers:
1. **Hardware / Emulation**: QEMU x86_64 virtual machine exposing serial COM1 (0x3F8), PIT (0x40/0x43), PIC (0x20/0xA0), and VGA text buffer (0xb8000).
2. **Bootloader Handover**: Limine protocol mapping the ELF kernel to `0xffffffff80000000` with paging enabled.
3. **Core Hardware Drivers**: Port I/O (`inb`/`outb`), UART 16550 serial driver, and VGA writer.
4. **Kernel Core**: GDT, IDT exception/interrupt gates, and structured logging.
5. **Memory Subsystem**: 128 KiB static kernel heap managed by `BumpAllocator`.
6. **Synchronization**: `SpinLock<T>` and `SpinLockGuard<'a, T>`.
7. **Task & Scheduler Subsystem**: `Task`, `ReadyQueue`, and `RoundRobinScheduler` with `DynamicQuantumPolicy`.
8. **Telemetry & Python Analysis**: Serial output streaming to `traces/qemu_serial.log` parsed by `scripts/analyze_traces.py`.

---

## 6. Design Decisions
- **`#![no_std]` and `#![no_main]`**: Complete detachment from the C runtime and Rust standard library, ensuring full bare-metal sovereignty.
- **Dual Compilation Targets**: Conditional compilation (`#[cfg(target_os = "none")]` vs `#[cfg(not(target_os = "none"))]`) allows the exact same data structures and algorithms to be tested both on the host using `cargo test` and bare-metal inside QEMU.
- **Ring Buffer Ready Queue**: Using a fixed-size array (`MAX_TASKS = 16`) eliminates dynamic memory allocation in the scheduling hot path, achieving deterministic $O(1)$ operations with zero allocation failure risk.
- **Interrupt Masking Spinlock**: Disables CPU interrupts upon lock acquisition to prevent deadlocks when interrupt service routines (ISRs) access shared kernel singletons.

---

## 7. Rust Implementation Details
The codebase strictly adheres to modern, idiomatic Rust:
- **Zero Raw Pointers in Public API**: All raw pointers (`*mut u8`, `*mut TaskContext`) are encapsulated inside safe Rust structs (`BumpAllocator`, `Task`).
- **RAII Resource Management**: Spinlocks release locks and restore interrupt flags automatically via the `Drop` trait implementation on `SpinLockGuard`.
- **Atomic Operations**: `AtomicBool` and `AtomicU64` with explicit `Ordering::Acquire`, `Ordering::Release`, and `Ordering::Relaxed`.

---

## 8. Task Management & Scheduling
Tasks are defined by the `Task` struct:
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
    pub current_quantum: u32,
    pub remaining_quantum: u32,
    pub context_switch_count: u32,
    pub yield_count: u32,
    pub preempt_count: u32,
    pub context: TaskContext,
    pub stack: [u8; DEFAULT_STACK_SIZE],
}
```
State transitions follow a strict finite state machine:
- `Ready -> Running`: Selected by `scheduler.schedule_next()`.
- `Running -> Ready`: Preempted by quantum expiration or voluntary yield.
- `Running -> Blocked`: Waiting on lock or event.
- `Blocked -> Ready`: Unblocked by resource availability.
- `Running -> Terminated`: Burst counter reached 0.

---

## 9. Specialization: Dynamic Time Quantum (DQ-RR)
The specialization dynamically adapts time slices using:

$$Q_{\text{dynamic}} = \text{clamp}\left( \left\lfloor \frac{\mathcal{T}_{\text{target}}}{\max(1, N)} \times \beta \times \frac{\text{Priority}}{5} \right\rfloor, Q_{\min}, Q_{\max} \right)$$

Where:
- $\mathcal{T}_{\text{target}} = 36$ ticks.
- $N$ is the number of runnable tasks.
- $\beta = 0.75$ for interactive tasks yielding cooperatively, $1.25$ for compute-bound tasks repeatedly exhausting quanta, and $1.0$ otherwise.
- Output is clamped to $[2, 16]$ ticks.

---

## 10. Timer and Interrupt Integration
The Intel 8254 PIT is programmed to Mode 3 (Square Wave Generator) at 100 Hz. The PIT divides the master oscillator frequency ($1,193,182 \text{ Hz}$) by a divisor of $11,931$:
$$\text{Divisor} = \frac{1,193,182}{100} \approx 11,931$$
On each tick:
1. `SYSTEM_TICKS` is atomically incremented.
2. The running task's remaining quantum and burst are decremented.
3. If quantum reaches 0, a preemption context switch is triggered.

---

## 11. Synchronization: `SpinLock<T>`
Implemented in `src/sync/spinlock.rs`:
```rust
pub struct SpinLock<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}
```
Acquisition uses atomic `compare_exchange_weak`:
```rust
while self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
    core::arch::asm!("pause");
}
```
Deadlock is prevented by capturing `RFLAGS` and clearing interrupts (`cli`) before spinning.

---

## 12. Memory Management: Bump Allocator
A 128 KiB static memory buffer is managed by `BumpAllocator`:
- **Allocation**:
  $$\text{start} = (\text{next} + \text{align} - 1) \& \sim(\text{align} - 1)$$
  $$\text{end} = \text{start} + \text{size}$$
  If $\text{end} \le \text{heap\_end}$, update $\text{next} = \text{end}$ and return $\text{start}$.
- **Safety Checks**: Rejects zero-sized requests (`ZeroSizedAllocation`), non-power-of-two alignments (`InvalidAlignment`), and out-of-bounds requests (`OutOfMemory`).

---

## 13. Testing and Validation
- **Host Unit Tests**: 15 automated unit tests covering task states, queue overflow, ring buffer wrap-around, dynamic quantum math, spinlock exclusion, and bump allocation limits.
- **Bare-Metal QEMU Runs**: Controlled execution of 4 heterogeneous tasks verifying scheduler switching, serial output, and clean exit via `isa-debug-exit`.

---

## 14. Experimental Results & Performance Analysis
From execution of the 4-task workload:
- **Total Workload Ticks**: 48 ticks (480 ms).
- **Context Switches**: 14 switches (compared to 18 in static RR).
- **Average Waiting Time**: 18.5 ticks.
- **Average Turnaround Time**: 30.5 ticks.
- **Interactive Task (T2) Turnaround**: 27 ticks vs 35 ticks in static RR (**22.8% improvement**).
- **Jain's Fairness Index**: **0.9124** (weighted), indicating highly equitable CPU distribution.

---

## 15. Limitations
1. Single-heap bump allocator lacks fine-grained individual `free()` capability.
2. Uniprocessor execution (no multi-core SMP load balancing).
3. Monolithic Ring 0 execution (no Ring 3 user space or page-fault isolation).
4. Ephemeral in-memory execution without persistent storage drivers.

---

## 16. Lessons Learned
- Low-level systems programming requires rigorous attention to hardware invariants (such as PIC vector offsets and interrupt-safe spinlocks).
- Rust's type system and ownership semantics drastically reduce concurrency and memory bugs in kernel space.
- Isolating platform-independent logic enables powerful host-side testing that accelerates development.

---

## 17. Future Work
- Implement a buddy allocator and paging virtual memory manager.
- Add Ring 3 user mode support with a `syscall` interface.
- Implement multicore SMP scheduling using APIC IPIs.
- Integrate an initramfs RAM-disk with a FAT/ext2 filesystem driver.

---

## 18. References
1. Silberschatz, A., Galvin, P. B., & Gagne, G. *Operating System Concepts* (10th ed.). Wiley.
2. Limine Bootloader Protocol Specification (v7). https://github.com/limine-bootloader/limine
3. Intel 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A: System Programming Guide.
4. Oppermann, P. *Writing an OS in Rust*. https://os.phil-opp.com/
5. Jain, R., Chiu, D. M., & Hawe, W. R. *A Quantitative Measure of Fairness and Discrimination for Resource Allocation in Shared Systems*. DEC-TR-301.

---

## 19. AI Usage Disclosure
Generative AI was used to assist with syntax exploration for Rust inline assembly (`core::arch::asm!`), formatting linker script syntax for Limine higher-half ELF loading, and drafting boilerplate Gradio UI layouts. All kernel algorithms (dynamic quantum formula, spinlock interrupt masking, ready queue ring buffer, bump allocator) were designed, reviewed, compiled, and verified directly on the system.

---

## 20. Individual Viva Defence Questions & Answers

### Q1: What happens immediately after the kernel starts?
**Answer:** The Limine bootloader enters `_start` in 64-bit Long Mode at virtual address `0xffffffff80000000`. The kernel initializes the 16550 UART serial port for debugging, reloads the GDT with kernel code (`0x08`) and data (`0x10`) selectors, remaps the 8259 PIC to vectors `0x20-0x2F`, loads the IDT, configures the 8254 PIT to 100 Hz, initializes the 128 KiB bump allocator heap, and spawns the initial tasks into the ready queue.

### Q2: How is a task represented?
**Answer:** A task is represented by a Task Control Block (`Task` struct) containing a unique `TaskId`, name, lifecycle state (`TaskState`), priority, remaining burst ticks, total runtime, waiting time, turnaround time, context switch count, dynamic quantum budget, an isolated 4 KiB stack, and a `TaskContext` struct holding saved CPU registers.

### Q3: How does the scheduler select the next task?
**Answer:** The scheduler uses Round Robin policy. It dequeues the next runnable task ID from the head of the `ReadyQueue` ring buffer in $O(1)$ time. If the previous task was preempted and still has remaining burst, it is placed at the tail of the queue.

### Q4: Why did you choose Round Robin?
**Answer:** Round Robin is fair, deterministic, starvation-free, and ideally suited for educational microkernels. Unlike FCFS, it prevents monopolization of the CPU by long processes. Unlike SJF, it does not require advance knowledge of burst times.

### Q5: What triggers a scheduling decision?
**Answer:** A scheduling decision is triggered by: (1) hardware timer tick expiration when the current task exhausts its remaining quantum; (2) task completion when remaining burst reaches 0; or (3) voluntary task yield when an interactive task finishes an event burst.

### Q6: What happens during a timer tick?
**Answer:** The 8254 PIT generates an IRQ0 interrupt, firing INT 0x20. The tick handler atomically increments `SYSTEM_TICKS`, increments the waiting time of all tasks in the ready queue, decrements the running task's burst and quantum, and checks if preemption or termination is required.

### Q7: How is shared kernel data protected?
**Answer:** Shared kernel data (such as the scheduler, allocator, and serial port) is protected by `SpinLock<T>`. Access is granted exclusively through an RAII `SpinLockGuard<'a, T>` that provides mutable dereferencing and automatically releases the lock upon drop.

### Q8: Why is synchronization required?
**Answer:** Even on a single-core CPU, preemptive timer interrupts can interrupt execution in the middle of a multi-instruction operation (e.g., updating queue head/tail pointers or bumping memory pointers). Without mutual exclusion, race conditions lead to pointer corruption, lost updates, and kernel crashes.

### Q9: How does your allocator work?
**Answer:** The `BumpAllocator` maintains a contiguous memory region with `heap_start`, `heap_end`, and `next` pointers. To allocate $N$ bytes with alignment $A$, it aligns `next` upwards to a multiple of $A$, verifies `next + N <= heap_end`, advances `next`, and returns the aligned starting address in $O(1)$ time.

### Q10: What happens when allocation fails?
**Answer:** If the requested allocation exceeds available heap memory, the allocator records a failed allocation in its telemetry statistics and returns `Err(AllocError::OutOfMemory)`. It does not panic or overwrite memory out of bounds.

### Q11: Where is `unsafe` used and why?
**Answer:** `unsafe` is used in: (1) `port.rs` for `inb`/`outb` CPU instructions; (2) `context.rs` for `switch_context` register assembly; (3) `spinlock.rs` to dereference `UnsafeCell`; (4) `bump.rs` for raw pointer arithmetic; and (5) `gdt.rs`/`idt.rs` for `lgdt`/`lidt` instructions. Every usage is strictly encapsulated behind a safe abstraction.

### Q12: What invariant does your `unsafe` code rely on?
**Answer:** In `switch_context`, it relies on the invariant that stack pointers are valid and 8-byte aligned. In `SpinLock`, it relies on the invariant that `UnsafeCell` is only dereferenced when the atomic boolean flag is held exclusively by the calling thread with interrupts masked. In `BumpAllocator`, it relies on the invariant that `next <= heap_end`.

### Q13: What happens if two tasks access the same resource?
**Answer:** The first task acquires the spinlock, setting `lock = true`. The second task attempts atomic compare-and-swap, fails, and executes `pause` in a spin loop until the first task drops its guard and resets `lock = false`.

### Q14: How did you test the scheduler?
**Answer:** Using a two-level strategy: Level 1 host automated unit tests in `tests/test_micro_os.rs` verifying FIFO queue order, boundary overflows, and dynamic quantum scaling; and Level 2 bare-metal QEMU execution with serial output verification.

### Q15: How did you establish that the scheduler is actually working?
**Answer:** By capturing the live serial trace (`traces/qemu_serial.log`) from QEMU and parsing it with `scripts/analyze_traces.py`. The trace demonstrates sequential context switch events, dynamic quantum adaptations, task preemption logs, and exact turnaround metrics matching theoretical calculations.

### Q16: How is your Micro-OS different from Linux?
**Answer:** Linux is a production monolithic kernel with millions of lines of code, multicore SMP scheduling (CFS/EEVDF), demand-paged virtual memory, user/kernel privilege separation (Ring 3 vs Ring 0), and extensive device drivers. Yashwanth Micro-OS is an educational microkernel (~1,500 lines) running in Ring 0 with single-core scheduling, a bump allocator, and focused visibility into core mechanisms.

### Q17: What did CA-5 teach you that helped this project?
**Answer:** CA-5 (Linux Systems Engineering) provided practical experience with CPU affinity, `strace`, scheduling latency, and kernel telemetry. This knowledge guided the design of our structured logging macros and Python trace analysis pipeline.

### Q18: What CA-4 research question could be extended into your Micro-OS?
**Answer:** A research investigation on *"Dynamic Time Quantum Tuning in Mixed Interactive/Compute Workloads"* was directly extended into the kernel's core specialization: implementing behavior-aware and queue-depth-aware quantum scaling.

### Q19: What would you implement next?
**Answer:** A page-frame allocator and x86_64 4-level paging to implement isolated user-space processes running in Ring 3 with system calls.

### Q20: What is the most important limitation of your current system?
**Answer:** The lack of Ring 3 hardware privilege separation and demand paging, meaning all tasks currently execute in supervisor mode without memory isolation boundaries.
