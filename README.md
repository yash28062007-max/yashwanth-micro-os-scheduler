# Yashwanth Micro-OS: Bare-Metal Educational Operating System (x86_64)

[![Language](https://img.shields.io/badge/Language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Target Architecture](https://img.shields.io/badge/Architecture-x86__64-blue.svg)]()
[![Target Spec](https://img.shields.io/badge/Target-x86__64--unknown--none-red.svg)]()
[![Bootloader](https://img.shields.io/badge/Bootloader-Limine%20v7-green.svg)](https://github.com/limine-bootloader/limine)
[![Emulators](https://img.shields.io/badge/Validated-QEMU%20%7C%20VirtualBox-purple.svg)]()
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Yashwanth Micro-OS** is an educational, bare-metal operating system kernel written in safe, freestanding **Rust (`no_std`, `no_main`)** for the **x86_64** architecture. Engineered for **Continuous Assessment 6 (CA-6)**, the kernel demonstrates fundamental operating system primitives, featuring a **genuine hardware timer interrupt subsystem (Intel 8254 PIT / 8259 PIC / IDT Vector 0x20)**, an interrupt-safe **`SpinLock<T>`**, a deterministic **Bump Allocator**, a graphical **Linear Framebuffer Console**, and an adaptive **Dynamic Quantum Round Robin (DQ-RR)** scheduler.

---

## 1. Architectural Overview & System Model

Yashwanth Micro-OS operates in 64-bit Long Mode within a higher-half virtual address space (`0xffffffff80000000`) established by the Limine bootloader protocol.

```
+------------------------------------------------------------------------+
|                      Yashwanth Micro-OS Kernel                         |
+------------------------------------------------------------------------+
|  [Task Subsystem]             [Scheduler Subsystem]                    |
|   - Task Control Block (TCB)   - ReadyQueue<16> Ring Buffer (O(1))     |
|   - Task States & Counters     - Dynamic Quantum Policy (DQ-RR)        |
+------------------------------------------------------------------------+
|  [Memory Subsystem]           [Synchronization]                        |
|   - 128 KiB Bump Allocator     - SpinLock<T> with AtomicBool           |
|   - 8/16-byte Alignment        - RAII SpinLockGuard (cli/pushfq)       |
+------------------------------------------------------------------------+
|  [Hardware Interrupt Subsystem]                                        |
|   - Global Descriptor Table (GDT: CS=0x08, DS=0x10)                   |
|   - 8259 PIC Remap (Master 0x20..0x27, Slave 0x28..0x2F)               |
|   - IDT Vector 32 (0x20): timer_interrupt_entry Naked Assembly ISR     |
|   - Intel 8254 PIT (100 Hz Rate Generator on Ports 0x40/0x43)          |
|   - Race-Free sti; hlt CPU Pacing & Lock-Free PIC EOI Acknowledgment   |
+------------------------------------------------------------------------+
|  [Console & Video Drivers]                                             |
|   - Linear Framebuffer Console (Double-buffered, 8x16 Bitmap Font)     |
|   - 16550 UART Serial (COM1 0x3F8, 38400 baud, 8N1 FIFO)               |
|   - Unbuffered Port I/O (inb, outb, inw, outw, io_wait)               |
+------------------------------------------------------------------------+
                                    |
                +-------------------+-------------------+
                |                                       |
      [Limine Bootloader v7]                  [Virtual Platforms]
   - Higher-Half Direct Map (HHDM)         - Headless QEMU x86_64
   - 64-bit Long Mode Handover             - Oracle VM VirtualBox (BIOS)
```

---

## 2. Key Features

1. **Bare-Metal Rust:** Freestanding `#![no_std]` and `#![no_main]` with compiler built-ins (`memcpy`, `memset`, `memmove`, `memcmp`) and red-zone deactivation (`-C no-redzone`).
2. **Dual Console Display:** Concurrently mirrors all diagnostics to both a 16550 UART serial port (COM1 `0x3F8`) and an on-screen graphical linear framebuffer console (with volatile pointer reads).
3. **Genuine Hardware Timer Interrupts:**
   - 8254 PIT generates IRQ0 pulses at 100 Hz (10 ms per tick).
   - 8259 PIC remapped to vectors `0x20`–`0x2F`; IRQ0 unmasked (`0xFE`).
   - IDT Vector 32 hooked via naked assembly ISR with dynamic 16-byte stack alignment and caller `RBP` preservation.
   - Lock-free, zero-allocation ISR callback (`on_timer_tick_irq()`) with Master PIC EOI.
   - Race-free CPU pacing via the x86 `sti; hlt` interrupt shadow.
4. **Adaptive Dynamic Quantum (DQ-RR):**
   - Automatically adapts time slices based on ready queue depth:
     $$Q = \text{clamp}\left( \left\lfloor \frac{36}{N_{\text{runnable}}} \times \beta \times \frac{\text{Priority}}{5} \right\rfloor, 2, 16 \right)$$
   - Rewards interactive tasks yielding cooperatively ($\beta = 0.75$); expands compute-bound slices ($\beta = 1.25$).
   - Avoids redundant self-switching when only one runnable task remains.
5. **Interrupt-Safe `SpinLock<T>`:** Atomic `compare_exchange_weak` with automatic interrupt masking (`cli`/`pushfq`) and RAII guard restoration.
6. **Deterministic Memory Allocator:** 128 KiB bump allocator with power-of-two alignment, bounds checking, and out-of-memory guards.
7. **Empirically Validated:** Full functional parity across QEMU and live VirtualBox BIOS mode (27 hardware timer ticks, 6 context switches, 0 faults).

---

## 3. Directory Layout

```
yashwanth-micro-os/
├── Cargo.toml                  # Kernel package configuration & dependencies
├── linker.ld                   # Higher-half linker script (0xffffffff80000000)
├── limine.conf                 # Bootloader menu configuration
├── .gitignore                  # Git ignore rules
├── src/
│   ├── lib.rs                  # Module exports, hlt_loop, and exit_qemu
│   ├── main.rs                 # Kernel entry point (_start), hardware setup, tasks
│   ├── kernel/                 # GDT, IDT, PIC, ISR stubs, logging, panic
│   ├── drivers/                # Framebuffer console, font, 16550 UART, port I/O
│   ├── task/                   # Task Control Block, lifecycle states, context
│   ├── scheduler/              # ReadyQueue, RoundRobin, DynamicQuantumPolicy
│   ├── timer/                  # Intel 8254 PIT, tick accounting, pending ticks
│   ├── sync/                   # SpinLock and RAII SpinLockGuard
│   └── memory/                 # Bump allocator and alignment logic
├── tests/
│   ├── test_micro_os.rs        # 17 automated unit tests (TC1–TC6 coverage)
│   ├── verify_implementation.py # 62 algorithmic verification checks
│   └── test_hf_app.py          # Hugging Face deployment test
├── docs/
│   ├── technical-report.md     # Comprehensive 20-section technical report
│   ├── testing.md              # CA-6 test case mapping (TC1–TC6)
│   ├── architecture.md         # Subsystem architecture & data flow
│   ├── design.md               # Design decisions & trade-offs
│   ├── limitations.md          # Architectural boundaries & future work
│   └── debugging-log.md        # Incident investigations and fixes
├── results/
│   ├── vbox_timer_screen.png   # Verified VirtualBox live screenshot
│   ├── vbox_debug_serial.log   # Complete VirtualBox COM1 serial log
│   └── performance_metrics.json# Exported benchmark metrics
├── build/
│   ├── yashwanth_micro_os.iso  # Final bootable hybrid ISO
│   └── yashwanth_micro_os_working_backup.iso # Preserved backup ISO
└── scripts/
    ├── build_iso.sh            # Rebuild kernel and ISO using xorriso
    ├── build_kernel.sh         # Bare-metal cargo build
    ├── run_qemu.sh             # Launch QEMU and capture serial traces
    └── analyze_traces.py       # Python metric and fairness analyzer
```

---

## 4. Build & Testing Instructions

### Prerequisites
- **Rust Toolchain:** Stable Rust with `x86_64-unknown-none` target (`rustup target add x86_64-unknown-none`).
- **Build Tools:** `xorriso`, `gcc` (for Limine host utility), `bash`.
- **Emulators:** `qemu-system-x86_64` or Oracle VM VirtualBox 7.x.

### 1. Run Host Unit Tests (17/17 Passing)
```bash
cargo test
```

### 2. Compile Kernel for Bare Metal
```bash
cargo check --target x86_64-unknown-none --features kernel_bin
```

### 3. Build Bootable Hybrid ISO
```bash
bash scripts/build_iso.sh
```
*Output: `build/yashwanth_micro_os.iso` (2.5 MB hybrid BIOS/UEFI bootable ISO).*

---

## 5. Execution Instructions

### Running in QEMU (Headless with Telemetry)
```bash
bash scripts/run_qemu.sh
```
Or directly:
```bash
qemu-system-x86_64 \
    -cpu max \
    -boot d \
    -cdrom build/yashwanth_micro_os.iso \
    -serial stdio \
    -display none \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -m 256M \
    -no-reboot
```

### Running in VirtualBox (BIOS Mode)
1. Create a VM: Type `Other / Unknown (64-bit)`, 2048 MB RAM, 1–2 CPUs.
2. In **System $\to$ Motherboard**, ensure **Enable EFI** is **unchecked** (BIOS mode).
3. In **Storage**, attach `build/yashwanth_micro_os.iso` to the IDE controller.
4. (Optional) In **Ports $\to$ Serial Ports**, enable COM1 in `Raw File` mode pointing to `results/vbox_debug_serial.log`.
5. Start the VM. The Limine bootloader will boot automatically into the graphical linear framebuffer.

---

## 6. CA-6 Test Cases (TC1–TC6)

- **TC1 (Minimum Configuration):** Verified scheduler switches between 2 tasks without starvation (`test_round_robin_preemption_and_switch`).
- **TC2 (Heterogeneous Tasks):** 4 concurrent tasks (CPU-bound, interactive yields, synchronization workers, diagnostics) executed to completion in QEMU and VirtualBox.
- **TC3 (Boundary Conditions):** Queue capacity limits (`MAX_TASKS = 16`), single-runnable-task yield continuation, single-tick burst completion, zero-size memory allocation rejection (`test_ready_queue_boundary_and_overflow`, `test_yield_when_single_runnable_task_avoids_self_switch`).
- **TC4 (Stress & Scalability):** Continuous circular queue wrap-around and context switching across 17 automated tests.
- **TC5 (Failure Handling):** Invalid state transition rejection, heap out-of-memory guards, and non-blocking `try_lock()` contention.
- **TC6 (Determinism & Repeatability):** 100% deterministic match in scheduling ticks (27), context switches (6), and turnaround metrics across repeated executions.

---

## 7. Performance & Verification Metrics

```text
================================================================
                 SCHEDULER PERFORMANCE REPORT                   
================================================================
TID    NAME                 PRIO   RUNTIME  WAITING  TURNAROUND CSW    PREEMPT
----------------------------------------------------------------
1      Task-1-CPUBound      5      18       23       41         2      1     
2      Task-2-Interactive   5      10       39       49         2      0     
3      Task-3-SyncWorker    8      14       12       26         1      0     
4      Task-4-Diagnostics   3      6        26       32         1      0     
----------------------------------------------------------------
Completed Tasks:           4
Total Context Switches:    6
Total Scheduling Ticks:    27
Total Hardware IRQ0 Ticks: 27
Average Waiting Time:      25 ticks (250 ms)
Average Turnaround Time:   37 ticks (370 ms)
Scheduling Efficiency:     100% (No starvation observed)
Jain's Fairness Index:     0.8780 (High fairness)
================================================================
```

---

## 8. Limitations & Execution Model Transparency

- **Hardware Subsystem:** Fully implemented and active (8254 PIT at 100 Hz, 8259 PIC remapped to `0x20`/`0x28`, IDT vector 32 hooked via naked assembly ISR, lock-free PIC EOI, race-free `sti; hlt` pacing).
- **Scheduling Policy:** Preemptive Dynamic Quantum Round Robin.
- **Execution Model Boundary:** Tasks execute as controlled sequential algorithmic burst profiles within the kernel thread rather than independent Ring 3 processes with separate stacks and asynchronous interrupt-frame context replacement (`RIP`/`RSP` swapping in `iretq`).

---

## 9. Author & License

- **Author:** Yashwanth (Operating Systems CA-6)
- **License:** MIT License. See [LICENSE](LICENSE) for details.
