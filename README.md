# Yashwanth Micro-OS: Bare-Metal Educational Operating System (x86_64)

[![Language](https://img.shields.io/badge/Language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Target](https://img.shields.io/badge/Architecture-x86__64-blue.svg)]()
[![Bootloader](https://img.shields.io/badge/Bootloader-Limine-green.svg)](https://github.com/limine-bootloader/limine)
[![Emulator](https://img.shields.io/badge/Emulator-QEMU-purple.svg)](https://www.qemu.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**Yashwanth Micro-OS** is an educational bare-metal operating system kernel written in safe, idiomatic **Rust** for the **x86_64** architecture. Built as the capstone engineering project for Continuous Assessment 6 (CA-6), the kernel focuses on **Scheduler Engineering** with a specialized **Dynamic Quantum Round Robin (DQ-RR)** scheduling algorithm.

---

## 1. Key Features & Subsystems

1. **Bootable Rust Kernel**: Loads directly into 64-bit Long Mode via the modern Limine bootloader protocol.
2. **Kernel Console & Structured Logging**: 16550 UART serial driver (`COM1 0x3F8`) and standard IBM VGA text mode (`0xb8000`) with structured logging macros (`log_info!`, `log_sched!`, `log_sync!`, `log_mem!`).
3. **Task Abstraction & Lifecycle**: Complete Task Control Block (TCB) implementation with `Ready`, `Running`, `Blocked`, and `Terminated` states.
4. **Deterministic Ready Queue**: Fixed-capacity ring buffer queue ($O(1)$ operations, zero dynamic allocations during scheduling).
5. **Specialization: Dynamic Quantum Round Robin**:
   - Adapts time slice based on ready queue depth: $Q = \text{clamp}(T_{\text{target}} / N, Q_{\min}, Q_{\max})$.
   - Adjusts for task execution behavior (rewards cooperative yields for interactive tasks; optimizes burst slices for compute-bound tasks).
   - Priority-weighted scaling.
6. **Hardware Timer & Interrupts**: Intel 8254 PIT generating 100 Hz timer ticks, 8259 PIC vector remapping, and custom Interrupt Descriptor Table (IDT).
7. **Interrupt-Safe Synchronization**: Custom `SpinLock<T>` with atomic flags (`AtomicBool`) and RAII `SpinLockGuard<'a, T>`.
8. **Kernel Memory Management**: Bounded Bump (Linear) Allocator with strict alignment handling and telemetry statistics.
9. **Companion Hugging Face Space**: Interactive Python/Gradio scheduler simulator for visual algorithm exploration.

---

## 2. Directory Layout

```
yashwanth-micro-os/
├── Cargo.toml                  # Kernel package definition
├── linker.ld                   # Linker script for higher-half (-2GiB) ELF
├── limine.conf                 # Bootloader menu configuration
├── src/
│   ├── lib.rs                  # Subsystem integration (no_std / host test cfg)
│   ├── main.rs                 # Bare-metal kernel entry point (_start)
│   ├── kernel/                 # GDT, IDT, PIC, Limine protocol, panic handler
│   ├── drivers/                # 16550 UART serial, VGA text mode, port I/O
│   ├── task/                   # Task representation, states, context switching
│   ├── scheduler/              # Ready queue, Round Robin, Dynamic Quantum
│   ├── timer/                  # Intel 8254 PIT, tick accounting
│   ├── sync/                   # SpinLock and RAII guard
│   └── memory/                 # Bump allocator and alignment logic
├── tests/
│   └── test_micro_os.rs        # Automated host-side test suite
├── docs/
│   ├── architecture.md         # Architecture specification & data flow
│   ├── design.md               # Design decisions & trade-offs
│   ├── testing.md              # Test plan, TC1-TC6 mapping
│   ├── debugging-log.md        # Real bug investigations & resolutions
│   ├── performance-analysis.md # Quantitative benchmark comparisons
│   ├── limitations.md          # Current limitations vs Linux
│   └── technical-report.md     # 20-section formal report & viva Q&A
├── scripts/
│   ├── setup_env.sh            # Safe environment verification script
│   ├── build_iso.sh            # Kernel compilation & ISO packaging
│   ├── run_qemu.sh             # QEMU execution with serial trace capture
│   ├── run_tests.sh            # Test suite runner
│   └── analyze_traces.py       # Python serial trace parser & fairness metrics
├── hf_space/                   # Companion Hugging Face Space (Gradio)
│   ├── app.py                  # Interactive scheduler simulator web app
│   ├── requirements.txt        # Python dependencies
│   └── README.md               # Hugging Face Spaces metadata
├── traces/                     # Captured execution logs
└── results/                    # Performance analysis JSON & test reports
```

---

## 3. Quick Start & Execution Guide

### Prerequisites
- Ubuntu 22.04 / 24.04 (WSL 2 or native Linux)
- Rust toolchain (`rustc`, `cargo`, `rust-src`)
- `qemu-system-x86`, `xorriso`, `mtools`, `python3`

### Step 1: Run Automated Host-Side Tests
```bash
cargo test --verbose
```
Runs 15 automated test cases validating task states, queue boundaries, dynamic quantum formulas, spinlocks, and memory allocators.

### Step 2: Build the Kernel & Boot in QEMU
```bash
# Build the kernel ELF
cargo build --release --target x86_64-unknown-none --bin yashwanth_micro_os

# Run the kernel in QEMU emulator
bash scripts/run_qemu.sh
```
The kernel boots in QEMU, initializes hardware subsystems, executes 4 concurrent tasks under the Dynamic Quantum Round Robin policy, outputs live telemetry to the serial console, prints the performance report, and cleanly exits via `isa-debug-exit`.

### Step 3: Analyze Captured Execution Traces
```bash
python3 scripts/analyze_traces.py traces/qemu_serial.log
```
Parses the raw serial trace, breaks down context switch reasons, and calculates Jain's Fairness Index.

---

## 4. Companion Hugging Face Space Simulator

To run the companion scheduler simulator locally:
```bash
cd hf_space
pip install -r requirements.txt
python app.py
```
Then visit `http://localhost:7860` to interactively adjust task burst times, queue capacities, and view the real-time Gantt scheduling timeline.

---

## 5. Quantitative Results Summary

| Metric | Static Round Robin ($Q=6$) | Dynamic Quantum (DQ-RR) | Improvement |
|---|---|---|---|
| **Interactive Turnaround (T2)** | 35 ticks | **27 ticks** | **22.8% faster** |
| **Context Switch Overhead** | 18 switches | **14 switches** | **22.2% reduction** |
| **Jain's Fairness Index** | 0.8841 | **0.9124** | **More equitable** |
| **CPU Starvation** | 0% | **0%** | **Guaranteed** |

---

## 6. License
MIT License. Developed for educational purposes as part of the Operating Systems CA-6 curriculum.
