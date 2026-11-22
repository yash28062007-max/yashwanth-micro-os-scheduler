# Yashwanth Micro-OS: Bare-Metal Educational Operating System (x86_64)

[![Language](https://img.shields.io/badge/Language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Target](https://img.shields.io/badge/Architecture-x86__64-blue.svg)]()
[![Target Spec](https://img.shields.io/badge/Target-x86__64--unknown--none-red.svg)]()
[![Bootloader](https://img.shields.io/badge/Bootloader-Limine%20v7-green.svg)](https://github.com/limine-bootloader/limine)
[![Emulator](https://img.shields.io/badge/Emulator-QEMU-purple.svg)](https://www.qemu.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Yashwanth Micro-OS** is an educational, bare-metal operating system kernel written in safe, freestanding **Rust (`no_std`, `no_main`)** for the **x86_64** architecture. Developed as the capstone engineering project for **Continuous Assessment 6 (CA-6)**, the kernel focuses on **Scheduler Engineering** featuring a **Dynamic Quantum Round Robin (DQ-RR)** scheduling algorithm.

---

## 1. Architectural Overview & System Model

Yashwanth Micro-OS is designed as a modular, single-address-space educational microkernel operating in supervisor mode (Ring 0) with a higher-half virtual address mapping (`0xffffffff80000000`) established by the Limine bootloader protocol.

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
|  [Core Kernel Architecture]                                            |
|   - Global Descriptor Table (GDT: CS=0x08, DS=0x10)                   |
|   - Interrupt Descriptor Table (IDT) & 8259 PIC Remap (0x20..0x2F)    |
|   - Intel 8254 PIT (100 Hz Rate Generator on Ports 0x40/0x43)          |
|   - Control Registers (CR0.MP/EM, CR4.OSFXSR/OSXMMEXCPT for SSE)       |
+------------------------------------------------------------------------+
|  [Hardware Drivers & Console]                                          |
|   - 16550 UART Serial (COM1 0x3F8, 38400 baud, 8N1 FIFO)               |
|   - Unbuffered Port I/O (inb, outb, inw, outw, io_wait)               |
|   - QEMU isa-debug-exit device (port 0xf4)                             |
+------------------------------------------------------------------------+
                                    |
               +--------------------+--------------------+
               |                                         |
     [Limine Bootloader v7]                    [QEMU x86_64 Emulator]
  - Higher-Half Direct Map (HHDM)           - Pure TCG / KVM Execution
  - 64-bit Long Mode Handover                - COM1 Serial Telemetry Log
```

### Execution & Concurrency Model
- **Execution Mode:** Ring 0 single-address-space educational kernel.
- **Scheduling Dispatch:** The kernel initializes the 8259 PIC and 8254 PIT hardware timer. The demonstration scheduling loop is driven by monotonic timer tick events (`increment_ticks()` and `on_timer_tick()`), evaluating dynamic time slices and voluntary cooperative yields.
- **Pedagogical Boundary:** True preemptive user-space multitasking (Ring 3 isolation with asynchronous hardware interrupt `iretq` stack frame swapping) is documented as an advanced architectural milestone for future research.

---

## 2. Key Features

1. **Bootable Rust Kernel**: Loads directly into 64-bit Long Mode via the modern Limine bootloader protocol (v7 specification).
2. **Console Diagnostics & Structured Logging**: 16550 UART serial driver on COM1 (`0x3F8`) with `fmt::Write`, `print!`, `println!`, and structured logging macros (`log_info!`, `log_sched!`, `log_mem!`, `log_sync!`, `log_task!`).
3. **Task Abstraction & Lifecycle**: Complete Task Control Block (TCB) tracking task ID, priority, burst times, runtime, waiting time, turnaround time, context switches, and 4 states (`Ready`, `Running`, `Blocked`, `Terminated`).
4. **Deterministic Ready Queue**: Fixed-capacity 16-slot circular ring buffer queue achieving deterministic $O(1)$ operations with zero dynamic memory allocations in the scheduling path.
5. **Specialization: Dynamic Quantum Round Robin**:
   - Adapts time slice based on ready queue depth: $Q = \text{clamp}(\lfloor \frac{T_{\text{target}}}{N_{\text{runnable}}} \times \beta \times \frac{\text{Priority}}{5} \rfloor, Q_{\min}, Q_{\max})$.
   - Rewards cooperative voluntary yields for interactive workloads; expands quanta during compute-heavy phases to minimize context-switching overhead.
6. **Hardware Timer & Descriptor Tables**: 64-bit flat GDT, 8259 PIC remapped to vectors `0x20-0x2F`, IDT, and Intel 8254 PIT configured to 100 Hz (10 ms per tick).
7. **Interrupt-Safe Synchronization**: Custom `SpinLock<T>` with atomic compare-and-swap (`AtomicBool`), interrupt masking (`cli`/`pushfq`), and RAII `SpinLockGuard`.
8. **Kernel Memory Management**: Bounded 128 KiB Bump Allocator with strict power-of-two alignment and out-of-memory guards.
9. **Companion Hugging Face Simulator**: Standalone Gradio 5/6 web application (`hf_space/app.py`) for visual algorithm exploration and Gantt chart generation.

---

## 3. Directory Layout

```
yashwanth-micro-os/
├── Cargo.toml                  # Kernel package configuration & dependencies
├── linker.ld                   # Higher-half linker script (0xffffffff80000000)
├── limine.conf                 # Bootloader menu configuration
├── .gitignore                  # Git ignore rules (build artifacts & venvs)
├── src/
│   ├── lib.rs                  # Module exports, hlt_loop, and exit_qemu
│   ├── main.rs                 # Kernel entry point (_start), SSE setup, tasks
│   ├── kernel/                 # GDT, IDT, PIC, Limine protocol, logging, panic
│   ├── drivers/                # 16550 UART serial, port I/O, VGA text buffer
│   ├── task/                   # Task representation, lifecycle states, context
│   ├── scheduler/              # ReadyQueue, RoundRobin, DynamicQuantumPolicy
│   ├── timer/                  # Intel 8254 PIT, tick accounting
│   ├── sync/                   # SpinLock and RAII SpinLockGuard
│   └── memory/                 # Bump allocator and alignment logic
├── tests/
│   ├── test_micro_os.rs        # 14 automated unit tests
│   ├── verify_implementation.py # 62 algorithmic and boundary checks
│   └── test_hf_app.py          # Automated Hugging Face simulator test
├── docs/
│   ├── demonstration-guide.md  # Step-by-step evaluator demonstration script
│   ├── architecture.md         # Subsystem architecture & data flow
│   ├── design.md               # Design trade-offs & design decisions
│   ├── testing.md              # Test strategy and test case mapping
│   ├── debugging-log.md        # 9 real engineering incidents and resolutions
│   ├── performance-analysis.md # Metric comparisons and fairness analysis
│   ├── limitations.md          # Systems engineering limitations vs Linux
│   └── technical-report.md     # 20-section formal report with Viva Q&A
├── scripts/
│   ├── setup_env.sh            # Safe environment verification script
│   ├── build_kernel.sh         # Cargo bare-metal compilation script
│   ├── build_iso.sh            # Bootable hybrid ISO generation with xorriso
│   ├── run_qemu.sh             # QEMU emulator execution & trace capture
│   └── analyze_traces.py       # Serial trace log parser & JSON exporter
├── hf_space/                   # Companion Hugging Face Space (Gradio)
│   ├── app.py                  # Interactive Gradio simulator web application
│   ├── requirements.txt        # Python dependencies (gradio, pandas, matplotlib)
│   └── README.md               # Hugging Face Spaces metadata
├── traces/                     # Serial logs captured during QEMU execution
└── results/                    # Exported JSON performance metrics
```

---

## 4. Installation & Build Instructions

### Prerequisites
- **OS:** Ubuntu 22.04 / 24.04 LTS (native or WSL2)
- **Rust:** `rustc` $\ge 1.75$ (tested with `rustc 1.100.0`)
- **Target:** `rustup target add x86_64-unknown-none`
- **Packages:** `qemu-system-x86`, `xorriso`, `mtools`, `python3`, `python3-venv`

```bash
# In Ubuntu / WSL2:
sudo apt-get update
sudo apt-get install -y qemu-system-x86 xorriso mtools python3 python3-venv
rustup default stable
rustup target add x86_64-unknown-none
```

### Step 1: Run Unit Tests
```bash
cargo test --verbose
```
Runs 14 automated unit tests validating task states, queue boundaries, dynamic quantum formulas, spinlocks, and heap allocation.

### Step 2: Run Algorithmic Verification
```bash
python3 tests/verify_implementation.py
```
Runs 62 automated checks validating file syntax, ready queue ring buffer operations, dynamic quantum calculations, and memory alignment.

### Step 3: Build the Bare-Metal Kernel Binary
```bash
cargo build --target x86_64-unknown-none --release --features kernel_bin
```
Produces the freestanding 64-bit ELF binary at `target/x86_64-unknown-none/release/yashwanth_micro_os`.

### Step 4: Package the Bootable ISO
```bash
bash scripts/build_iso.sh
```
Compiles the kernel, prepares the ISO structure, packages the bootable hybrid ISO (`build/yashwanth_micro_os.iso`), and installs Limine BIOS stages.

---

## 5. Execution & Verification in QEMU

Run the kernel in QEMU with serial output captured to `traces/qemu_serial.log`:
```bash
bash scripts/run_qemu.sh
```

### Verified Kernel Execution Results
```text
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
```

### Automated Trace Analysis
```bash
python3 scripts/analyze_traces.py traces/qemu_serial.log
```
Extracts verified telemetry and computes **Jain's Fairness Index: 0.8780**.

---

## 6. Companion Hugging Face Space Simulator

To run the interactive Gradio scheduler simulator:
```bash
cd hf_space
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python app.py
```
Access the simulator in your browser at `http://localhost:7860`.

---

## 7. Documentation Index

- [Demonstration Guide](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/demonstration-guide.md): Step-by-step evaluation guide for examiners.
- [Academic Technical Report](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/technical-report.md): Formal 20-section report and Viva questions/answers.
- [Architecture Specification](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/architecture.md): Subsystems, memory maps, and data flow.
- [Design Decisions](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/design.md): Architectural trade-offs and rationale.
- [Testing Strategy](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/testing.md): Test suite mapping and coverage.
- [Debugging Log](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/debugging-log.md): Record of 9 resolved hardware and software incidents.
- [Performance Analysis](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/performance-analysis.md): Quantitative comparison between static and dynamic Round Robin.
- [Limitations & Future Work](file:///C:/Users/yash2/.gemini/antigravity/scratch/yashwanth-micro-os/docs/limitations.md): Current constraints and production kernel comparison.

---

## 8. License
MIT License. Developed for educational purposes for the Operating Systems CA-6 curriculum.
