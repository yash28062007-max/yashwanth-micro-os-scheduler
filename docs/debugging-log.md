# Yashwanth Micro-OS: Engineering Debugging Log

## Incident 1: 8259 PIC Interrupt Clash with CPU Exceptions
- **Observation**: Early prototype crashed immediately upon enabling interrupts with a General Protection Fault (vector 0x0D).
- **Hypothesis**: The default IBM PC 8259 PIC maps Master IRQ 0..7 to CPU interrupt vectors 0x08..0x0F. In protected/long mode, vectors 0x08..0x0F are reserved by the x86 architecture for double faults, invalid TSS, segment not present, and stack segment faults!
- **Root Cause**: The hardware timer tick (IRQ0) was firing on vector 0x08, triggering a double-fault exception handler instead of a timer ISR.
- **Resolution**: Implemented `remap_pic(0x20, 0x28)` in `src/kernel/idt.rs`. IRQ 0..7 now safely map to vectors 0x20..0x27, and IRQ 8..15 map to 0x28..0x2F.
- **Verification**: Timer ticks now dispatch cleanly on vector 0x20 without triggering CPU exception traps.

---

## Incident 2: SpinLock Deadlock under Hardware Interrupts
- **Observation**: Kernel hung intermittently during serial logging while a task was holding a spinlock.
- **Hypothesis**: A timer tick occurred while the CPU was executing inside `SERIAL_PORT.lock()`. The timer interrupt handler also attempted to log via serial, trying to re-acquire the same spinlock on the same CPU core, causing an infinite spin deadlock.
- **Root Cause**: `SpinLock` was not interrupt-safe.
- **Resolution**: Updated `SpinLock::lock()` in `src/sync/spinlock.rs` to read the CPU `RFLAGS` register and execute `cli` (clearing IF) before entering the atomic compare-and-swap spin loop. In `SpinLockGuard::drop()`, the previous interrupt flag is restored.
- **Verification**: Verified under heavy multi-task logging; zero deadlocks observed.

---

## Incident 3: Division by Zero in Dynamic Quantum Computation
- **Observation**: Panic occurred during task state transition when the ready queue was temporarily drained.
- **Hypothesis**: The dynamic quantum formula `T_target / N_runnable` attempted division by zero when `runnable_count == 0`.
- **Root Cause**: Unchecked integer division when `runnable_count` evaluated to 0.
- **Resolution**: Added guard in `DynamicQuantumPolicy::compute_quantum`:
  ```rust
  let n = if runnable_count == 0 { 1 } else { runnable_count as u32 };
  ```
- **Verification**: Added boundary test `test_dynamic_quantum_calculation` with empty queue; verified stable quantum fallback to `max_quantum`.

---

## Incident 4: WSL Package Resolution (IPv6 Unreachable Route)
- **Observation**: `apt-get` inside WSL timed out trying to connect to Ubuntu archive servers on IPv6 addresses `(2620:2d:4002:1::103)`.
- **Hypothesis**: WSL2 virtual network adapter had IPv6 DNS resolution enabled, but the Windows host ISP only routes IPv4.
- **Root Cause**: Default apt behavior attempts IPv6 prior to falling back to IPv4, causing long connection delays and socket errors.
- **Resolution**: Configured `/etc/apt/apt.conf.d/99force-ipv4` with `Acquire::ForceIPv4 "true";` to force direct IPv4 connections.
- **Verification**: Package updates and installations succeeded immediately via IPv4 without socket drops.

---

## Incident 5: Limine Bootloader Protocol v7 Magic Numbers
- **Observation**: Limine displayed `limine: Requests count: 0` and failed to pass HHDM and memory map information to the kernel.
- **Hypothesis**: The magic numbers used in the protocol header structs did not match the specification expected by Limine v7.x.
- **Root Cause**: Limine v7 protocol requires exact 64-bit magic arrays (`0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 2` for Base Revision, `0xc7b1dd30df4c8b88, 0x0a82e883a194f07b` for Common Request Magic), as well as `.limine_requests_start` and `.limine_requests_end` 4-quadword and 2-quadword boundary markers.
- **Resolution**: Updated `src/kernel/limine_proto.rs` and `linker.ld` to define the exact Limine v7 structs and section placement.
- **Verification**: Limine detected 2 requests, reported base revision 2, and placed the higher-half direct map at `0x100000000`.

---

## Incident 6: GNU ld Bitcode Incompatibility with Release LTO
- **Observation**: Compiling with `lto = true` in `[profile.release]` caused `ld: libyashwanth_micro_os.rlib: error adding symbols: file format not recognized`.
- **Hypothesis**: Rust's link-time optimization emits LLVM bitcode inside `.rcgu.o` object files rather than native ELF machine code, which GNU `ld` cannot link without the LLVM gold plugin.
- **Root Cause**: `lto = true` in `Cargo.toml`.
- **Resolution**: Set `lto = false` and `codegen-units = 1` in `Cargo.toml`.
- **Verification**: GNU `ld` directly linked the library archive without symbol or format errors.

---

## Incident 7: Missing Freestanding Memory Intrinsics
- **Observation**: Linking the bare-metal kernel with `-nostdlib` produced undefined references to `memcpy`, `memset`, `memmove`, and `memcmp`.
- **Hypothesis**: In `no_std` environments, LLVM lowering can generate calls to standard C memory intrinsics for structure assignments and slice comparisons. Without libc, these symbols must be provided by the kernel.
- **Root Cause**: Absence of compiler-builtins or libc in freestanding bare-metal mode.
- **Resolution**: Implemented freestanding `#[no_mangle] pub unsafe extern "C"` functions for `memcpy`, `memset`, `memmove`, and `memcmp` in `src/main.rs`.
- **Verification**: Binary linked cleanly without external C library dependencies.

---

## Incident 8: QEMU TCG 1GiB Huge Page Support
- **Observation**: Limine halted early during boot with `paging: 1GiB pages are not supported!`.
- **Hypothesis**: Limine v7 uses 1 GiB huge pages (`pdpe1gb` CPU flag) for higher-half direct physical memory mapping. The default QEMU TCG emulation CPU model does not advertise 1GiB page capability.
- **Root Cause**: Default QEMU CPU flag omission.
- **Resolution**: Configured QEMU with `-cpu max` in `scripts/run_qemu.sh`.
- **Verification**: Limine successfully initialized paging: `paging: 1GiB pages are supported!`.

---

## Incident 9: General Protection Fault (#GP error 0x0032) on SIMD `xorps`
- **Observation**: Kernel triple-faulted immediately after serial port initialization inside `_print`.
- **Hypothesis**: Disassembly with `objdump` revealed the fault occurred at instruction `0f 57 c0 (xorps %xmm0, %xmm0)`. Register dump showed `CR4=0x00000020`, meaning CR4.OSFXSR (bit 9) was 0.
- **Root Cause**: In 64-bit mode, executing SSE/SIMD instructions when CR4.OSFXSR is disabled triggers a General Protection Fault (#GP). Rust's compiler generates 128-bit SIMD instructions to zero stack memory during `format_args!` and struct initialization.
- **Resolution**: Added low-level control register setup in `_start()` before any formatted printing:
  - Cleared CR0.EM (bit 2) and set CR0.MP (bit 1).
  - Set CR4.OSFXSR (bit 9) and CR4.OSXMMEXCPT (bit 10).
- **Verification**: Kernel booted, initialized formatted serial logging, and completed the full scheduling trace without CPU exceptions.
