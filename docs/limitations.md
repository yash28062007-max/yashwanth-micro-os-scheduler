# Yashwanth Micro-OS: System Limitations & Future Extensions

## 1. Overview

**Yashwanth Micro-OS** is an educational microkernel developed to make core operating system mechanisms visible, measurable, and verifiable. In keeping with the project philosophy:

> *"Do not build a large operating system. Build a small operating system that makes OS mechanisms visible."*

Certain components are intentionally bounded in scope. This document candidly details those limitations and contrasts them with a production operating system such as Linux.

---

## 2. Subsystem Analysis & Boundaries

### 2.1 Hardware Timer Interrupts vs. Task Thread Execution
- **Current Mechanism**: 
  - **Hardware Level:** Fully implemented and active. The Intel 8254 PIT generates physical IRQ0 interrupts at 100 Hz (10 ms per tick). The 8259 PIC is remapped to vectors `0x20`–`0x2F`, vector 32 (`0x20`) is registered in the IDT, and the naked assembly ISR `timer_interrupt_entry` dynamically aligns the stack to 16 bytes, saves caller `RBP` and registers, increments `SYSTEM_TICKS` and `HW_INTERRUPT_COUNT` via lock-free atomics, acknowledges the PIC with `send_eoi(0)`, and returns via `iretq`.
  - **Scheduler Level:** The Round Robin policy enforces preemptive time slice decisions (`SwitchReason::QuantumExpired`).
  - **Task Execution Boundary:** The benchmark tasks execute as controlled sequential algorithmic burst profiles paced by the hardware timer, rather than independent threads with isolated Ring 3 user stacks and asynchronous interrupt-frame context replacement (`RIP`/`RSP` swapping in `iretq`).
- **Linux Comparison:** Linux uses timer interrupts (`CONFIG_HZ`) and `scheduler_tick()` to set `TIF_NEED_RESCHED`, forcibly preempting arbitrary user-mode and kernel-mode instructions upon returning from the interrupt frame.
- **Future Direction:** Allocate independent 16 KiB stacks per task in dynamic memory and implement full thread preemption inside the timer ISR by replacing the `iretq` stack frame pointers.

### 2.2 Memory Management
- **Current Mechanism**: Single-heap Bump (Linear) Allocator (128 KiB).
- **Limitation**: The allocator cannot reclaim or coalesce individual deallocated memory blocks. Memory is allocated monotonically and can only be freed in bulk by resetting the heap (`allocator.reset()`).
- **Linux Comparison:** The Linux kernel employs a multi-tiered memory architecture consisting of the Buddy Allocator (physical page frames), SLAB/SLUB allocators (kernel object caches), and demand-paged Virtual Memory Areas (VMAs).
- **Future Direction:** Implement a Free-List or Buddy allocator that tracks freed memory blocks using segregated free lists and bitmap page tracking.

### 2.3 Task Privilege & Isolation Boundary
- **Current Mechanism**: All tasks execute in **Ring 0 (Supervisor Mode)** within a single address space.
- **Limitation**: There is no hardware privilege separation between user space and kernel space. A fault or stack corruption in a task could compromise kernel memory.
- **Linux Comparison:** Linux enforces strict hardware privilege separation between Ring 3 (User Space) and Ring 0 (Kernel Space) using hardware page protection bits and the `syscall` / `sysret` instruction interface.
- **Future Direction:** Create Ring 3 TSS (Task State Segment) descriptors, per-task page tables (CR3 switching), and a minimal system call interface (`syscall`/`sysret`).

### 2.4 Single-Core Scheduling
- **Current Mechanism**: Uniprocessor Round Robin scheduling on CPU Core 0.
- **Limitation**: Multicore / Symmetric Multiprocessing (SMP) is not enabled; the kernel does not issue APIC Inter-Processor Interrupts (IPIs) to awaken secondary application processors (APs).
- **Linux Comparison:** The Linux Completely Fair Scheduler (CFS) and EEVDF scheduler manage per-CPU runqueues with active load balancing and CPU affinity masks.
- **Future Direction:** Parse ACPI MADT tables to discover secondary cores and initialize local APICs for multicore SMP scheduling.

### 2.5 File System & Storage
- **Current Mechanism**: Ephemeral in-memory execution.
- **Limitation**: The OS does not include a block device driver (AHCI/NVMe) or a persistent file system (such as ext4 or FAT32).
- **Future Direction:** Implement an initramfs RAM-disk driver with a read-only TAR or FAT16 filesystem parser.

---

## 3. Summary Comparison Matrix

| Operating System Feature | Linux Kernel (v6.x) | Yashwanth Micro-OS (CA-6) | Educational Purpose |
|---|---|---|---|
| **Language** | C & Rust (Drivers) | **100% Rust** | Idiomatic safe systems programming |
| **CPU Target** | Multi-architecture | **x86_64** | Focus on standard PC architecture |
| **Boot Protocol** | UEFI / GRUB / Linux boot | **Limine** | Clean entry into 64-bit Long Mode |
| **Timer Subsystem** | APIC Timer / HPET / TSC | **8254 PIT (100 Hz)** | Classic hardware timer architecture |
| **Interrupt Controller** | Local & I/O APIC / MSI-X | **Dual 8259 PIC Remapped** | Low-level PIC ICW initialization |
| **Scheduler** | EEVDF / CFS (Multi-core) | **Dynamic Quantum RR** | Queue-depth & behavior adaptive scaling |
| **Privilege Levels** | Ring 0 / Ring 3 Isolation | **Ring 0** | Direct visibility into kernel mechanisms |
| **Memory Allocator** | Buddy + SLUB + Demand Paging| **Bump Allocator** | Deterministic $O(1)$ allocation |
| **Synchronization** | RCU, Mutexes, Spinlocks | **AtomicBool SpinLock** | Core concept of mutual exclusion & IRQ safety |
| **Telemetry** | eBPF, `ftrace`, `dmesg` | **Structured Serial & Screen Logs**| Dual output with Python trace analysis |
