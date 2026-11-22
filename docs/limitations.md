# Yashwanth Micro-OS: System Limitations & Future Extensions

## 1. Overview

**Yashwanth Micro-OS** is an educational microkernel developed to make core operating system mechanisms visible and measurable. In keeping with the project philosophy:

> *"Do not build a large operating system. Build a small operating system that makes OS mechanisms visible."*

Certain components are intentionally simplified. This document candidly details those limitations and contrasts them with a production operating system such as Linux.

---

## 2. Identified Subsystem Limitations

### 2.1 Memory Management
- **Current Mechanism**: Single-heap Bump (Linear) Allocator.
- **Limitation**: The allocator cannot reclaim or coalesce individual deallocated memory blocks. Memory can only be freed in bulk by resetting the heap (`allocator.reset()`).
- **Linux Comparison**: The Linux kernel employs a multi-tiered memory architecture consisting of the Buddy Allocator (physical page frames), SLAB/SLUB allocators (kernel object caches), and demand-paged Virtual Memory Areas (VMAs).
- **Future Direction**: Implement a Free-List or Buddy allocator that tracks freed memory blocks using a segregated free list.

### 2.2 Task Privilege & Isolation Boundary
- **Current Mechanism**: All tasks execute in **Ring 0 (Supervisor Mode)**.
- **Limitation**: There is no separation between user space and kernel space. A bug or stack corruption in a task could potentially overwrite kernel data structures.
- **Linux Comparison**: Linux enforces hardware privilege separation between Ring 3 (User Space) and Ring 0 (Kernel Space) using paging write-protect bits and the `syscall` / `sysret` instruction interface.
- **Future Direction**: Create Ring 3 TSS (Task State Segment) descriptors, user page tables, and a minimal system call interface (`int 0x80` or `syscall`).

### 2.3 Single-Core Scheduling
- **Current Mechanism**: Uniprocessor Round Robin scheduling on CPU Core 0.
- **Limitation**: Multicore / Symmetric Multiprocessing (SMP) is not enabled; the kernel does not issue APIC Inter-Processor Interrupts (IPIs) to awaken secondary application processors (APs).
- **Linux Comparison**: The Linux Completely Fair Scheduler (CFS) and EEVDF scheduler manage per-CPU runqueues with active load balancing and CPU affinity masks.
- **Future Direction**: Parse ACPI MADT tables to discover secondary cores and initialize local APICs for multicore SMP scheduling.

### 2.4 File System & Storage
- **Current Mechanism**: Ephemeral in-memory execution.
- **Limitation**: The OS does not include an AHCI/NVMe disk driver or a persistent file system (such as ext4 or FAT32).
- **Future Direction**: Implement a RAM-disk (initramfs) driver or an AHCI SATA controller driver with a read-only TAR or FAT16 filesystem.

### 2.5 Scheduling Dispatch Mechanism & Preemption
- **Current Mechanism**: Monotonic timer tick dispatch loop with cooperative task yield evaluation.
- **Limitation**: While low-level hardware interrupt structures (GDT, IDT, 8259 PIC remapping, 8254 PIT rate generator at 100 Hz) are fully initialized on bare metal, the execution loop in `src/main.rs` dispatches ticks synchronously via `increment_ticks()` and `on_timer_tick()` rather than preempting running tasks via asynchronous hardware INT 0x20 ISR `iretq` stack frame swapping.
- **Linux Comparison**: The Linux kernel uses timer interrupts (`CONFIG_HZ`) and scheduler tick handlers (`scheduler_tick()`) to trigger asynchronous preemption (`TIF_NEED_RESCHED`) upon return from interrupt or syscall.
- **Future Direction**: Connect the IDT INT 0x20 entry to a naked assembly context-switch handler that saves all general-purpose registers to the task stack, updates the task's saved `rsp`, and executes an `iretq` after switching stack pointers.

---

## 3. Summary Comparison Matrix

| Operating System Feature | Linux Kernel (v6.x) | Yashwanth Micro-OS (CA-6) | Educational Purpose |
|---|---|---|---|
| **Language** | C & Rust (Drivers) | **100% Rust** | Idiomatic safe systems programming |
| **CPU Target** | Multi-architecture | **x86_64** | Focus on standard PC architecture |
| **Boot Protocol** | UEFI / GRUB / Linux boot | **Limine** | Clean entry into 64-bit Long Mode |
| **Scheduler** | EEVDF / CFS (Multi-core) | **Dynamic Quantum RR** | Demonstrates queue-depth & behavior scaling |
| **Privilege Levels** | Ring 0 / Ring 3 Isolation | **Ring 0** | Direct visibility into kernel mechanisms |
| **Memory Allocator** | Buddy + SLUB + Demand Paging| **Bump Allocator** | Deterministic $O(1)$ allocation |
| **Synchronization** | RCU, Mutexes, Spinlocks | **AtomicBool SpinLock** | Core concept of mutual exclusion & IRQ safety |
| **Telemetry** | eBPF, `ftrace`, `dmesg` | **Structured Serial Logs** | Easy parsing with Python analysis tools |
