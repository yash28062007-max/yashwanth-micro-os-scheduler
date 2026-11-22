# Yashwanth Micro-OS: Design Decisions & Implementation Rationale

## 1. Overview

This document presents the detailed engineering rationale and architectural trade-offs behind **Yashwanth Micro-OS**. Every design decision is justified through operating system principles, hardware constraints, and educational objectives.

---

## 2. Kernel Design Decisions

### 2.1 Language Selection: Why Rust?
Rust was selected as the implementation language for the Micro-OS because:
1. **Zero-Cost Abstractions**: Rust compiles down to bare-metal machine code without a runtime, garbage collector, or mandatory standard library (`#![no_std]`).
2. **Memory Safety & Concurrency**: The borrow checker prevents data races, use-after-free, and dangling pointer bugs at compile time.
3. **Explicit `unsafe` Isolation**: Hardware-level operations (port I/O, writing page tables, manipulating CPU registers) are explicitly partitioned into well-documented `unsafe` blocks, while the rest of the kernel is guaranteed safe.

### 2.2 Bootloader Selection: Why Limine?
- Limine is an open, modern, compliant bootloader protocol that enters the kernel directly in **64-bit Long Mode**.
- Unlike legacy Multiboot (which boots in 32-bit protected mode and requires extensive assembly trampolines to switch to 64-bit), Limine sets up paging, maps the kernel to the higher-half (`-2 GiB`), and provides a clean memory map.
- This allows the kernel code to begin directly in high-level Rust.

---

## 3. Scheduler Specialization: Dynamic Quantum Round Robin (DQ-RR)

### 3.1 The Limitation of Static Round Robin
In classical Round Robin scheduling, the time quantum $Q$ is constant. This creates an engineering dilemma:
- **Large Quantum ($Q \gg 10$)**: Average waiting time increases because tasks must wait for preceding tasks to complete lengthy slices. The scheduler degrades towards First-Come, First-Served (FCFS).
- **Small Quantum ($Q \le 2$)**: Interactive responsiveness is excellent, but frequent context switches waste CPU cycles saving and restoring registers.
- **Load Insensitivity**: If 2 tasks are running, a large quantum is fine. If 15 tasks are running, the time between slices ($N \times Q$) exceeds acceptable interactive bounds.

### 3.2 Dynamic Quantum Mathematical Derivation
Yashwanth Micro-OS introduces an adaptive dynamic quantum policy:

$$\mathcal{T}_{\text{cycle}} = N_{\text{runnable}} \times Q_{\text{avg}} \le \mathcal{T}_{\text{target}}$$

Where $\mathcal{T}_{\text{target}}$ is the desired cycle latency (configured to 36 ticks).

1. **Load-Adaptive Base Quantum**:
   $$Q_{\text{load}} = \text{clamp}\left( \left\lfloor \frac{\mathcal{T}_{\text{target}}}{\max(1, N_{\text{runnable}})} \right\rfloor, Q_{\min}, Q_{\max} \right)$$
   Where $Q_{\min} = 2$ ticks and $Q_{\max} = 16$ ticks.

2. **Workload Behavior Feedback**:
   Tasks record their historical preemption count ($C_{\text{preempt}}$) and voluntary yield count ($C_{\text{yield}}$):
   $$\beta = \begin{cases} 
   0.75 & \text{if } C_{\text{yield}} > C_{\text{preempt}} \quad (\text{Interactive / I/O-bound}) \\ 
   1.25 & \text{if } C_{\text{preempt}} > 0 \text{ and } C_{\text{yield}} = 0 \quad (\text{CPU-bound}) \\ 
   1.00 & \text{otherwise} 
   \end{cases}$$

3. **Priority Weighting**:
   $$P_{\text{factor}} = \frac{\text{Priority}}{5.0}$$

4. **Final Computed Quantum**:
   $$Q_{\text{dynamic}} = \text{clamp}\left( \lfloor Q_{\text{load}} \times \beta \times P_{\text{factor}} \rfloor, Q_{\min}, Q_{\max} \right)$$

This guarantees:
- CPU-bound tasks receive larger slices, reducing context-switch overhead.
- Interactive tasks that yield voluntarily receive shorter, high-frequency slices, reducing waiting latency.
- High queue occupancy automatically compresses quanta to maintain system responsiveness.

---

## 4. Synchronization Mechanism: `SpinLock<T>`

### 4.1 Design Choice: Spinlock vs Sleeping Mutex
In a bare-metal kernel prior to complete interrupt-driven thread blocking queues, a spinlock is the foundational synchronization primitive.

### 4.2 Deadlock Prevention: Interrupt Masking
If task execution or a kernel driver acquires a spinlock, and a hardware timer interrupt fires whose ISR also tries to acquire that spinlock, the CPU enters an unrecoverable **deadlock**.
To prevent this:
1. `SpinLock::lock()` reads the CPU `RFLAGS` register to record the interrupt flag (`IF`), then disables interrupts using `cli`.
2. The atomic exchange spins with a `pause` instruction (minimizing CPU pipeline contention).
3. `SpinLockGuard::drop()` stores `false` using `Ordering::Release` and restores the original interrupt state (`sti` if previously enabled).

---

## 5. Memory Allocator: Bump Allocator

### 5.1 Why Bump (Linear) Allocation?
- **Deterministic $O(1)$ Time**: Allocation is simply `next = align_up(next) + size`. No search loops or linked-list traversals.
- **Zero Internal Fragmentation Overhead**: No metadata headers are required between allocated blocks.
- **Suitability for Bootstrapping**: Operating systems frequently use a bump allocator during early initialization to allocate page tables, task stacks, and descriptor tables before complex heap allocators (like slab or buddy) are activated.

### 5.2 Limitations & Mitigations
- A pure bump allocator cannot free individual blocks.
- **Mitigation**: Yashwanth Micro-OS implements `BumpAllocator::reset()` for scenario recycling and maintains explicit observability metrics (`allocated_bytes`, `peak_allocated_bytes`, `failed_allocations`).

---

## 6. Unsafe Rust Encapsulation Policy

Every `unsafe` operation is constrained behind safe interfaces:
| Subsystem | `unsafe` Operation | Justification | Safe Invariant Enforced |
|---|---|---|---|
| `drivers/port.rs` | `inb`, `outb` inline asm | Direct x86 port I/O | Port number bounds checked; isolated in driver |
| `task/context.rs` | `switch_context` naked asm | Register saving / restoration | Stack pointer and frame alignment validated |
| `sync/spinlock.rs` | Dereferencing `UnsafeCell` | Mutual exclusion data access | Access only permitted while atomic lock is true |
| `memory/bump.rs` | Raw pointer arithmetic | Heap allocation | Pointer cannot exceed `heap_end` |
| `kernel/gdt.rs` | `lgdt`, segment reloading | Loading CPU descriptor tables | Static aligned GDT structures |
