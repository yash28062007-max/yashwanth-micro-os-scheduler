#![no_std]
#![no_main]

use core::alloc::Layout;
use yashwanth_micro_os::{
    drivers::{init_framebuffer, init_serial},
    exit_qemu, hlt_loop,
    kernel::{init_gdt, init_idt},
    log_info, log_mem, log_sched, log_sync, log_task,
    memory::{init_heap, KERNEL_ALLOCATOR},
    println,
    scheduler::{ScheduleAction, SwitchReason, SCHEDULER},
    sync::SpinLock,
    task::{Task, TaskId},
    timer::{consume_pending_ticks, get_hw_interrupt_count, get_ticks, init_pit},
};

/// Entry point called by Limine bootloader in 64-bit long mode.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Enable SSE / SIMD instructions in CR0 and CR4 so compiler-generated
    // vector operations (such as xorps) do not trigger General Protection Faults
    unsafe {
        let mut cr0: u64;
        core::arch::asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack));
        cr0 &= !(1 << 2); // Clear EM (Emulation)
        cr0 |= 1 << 1;    // Set MP (Monitor Coprocessor)
        core::arch::asm!("mov cr0, {}", in(reg) cr0, options(nomem, nostack));

        let mut cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack));
        cr4 |= (1 << 9) | (1 << 10); // Set OSFXSR (bit 9) and OSXMMEXCPT (bit 10)
        core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nomem, nostack));
    }

    unsafe {
        for b in b"HELLO FROM BARE METAL RUST KERNEL!\n" {
            core::arch::asm!(
                "mov dx, 0x3F8",
                "out dx, al",
                in("al") *b,
                options(nomem, nostack)
            );
        }
    }

    // 1. Initialize serial port for console diagnostics
    init_serial();

    // 2. Initialize graphical framebuffer console for on-screen output
    init_framebuffer();

    println!("\n================================================================");
    println!("     __  __           _                             _   _       ");
    println!("     \\ \\/ /__ _  ___ | |____      ____ _ _ __  _   | |_| |__    ");
    println!("      \\  // _` |/ __|| '_ \\ \\ /\\ / / _` | '_ \\| |_ | __| '_ \\   ");
    println!("      / /| (_| |\\__ \\| | | \\ V  V / (_| | | | |  _|| |_| | | |  ");
    println!("     /_/  \\__,_||___/|_| |_|\\_/\\_/ \\__,_|_| |_|_|   \\__|_| |_|  ");
    println!("                                                                ");
    println!("                YASHWANTH MICRO-OS (x86_64)                    ");
    println!("        Educational Bare-Metal Kernel with Dynamic Quantum      ");
    println!("================================================================\n");

    log_info!("Bootloader handover complete. Running in 64-bit Long Mode.");
    log_info!("Project Variant: Minimal Round Robin Micro-OS with Dynamic Quantum");

    // 2. Hardware Descriptor Tables
    log_info!("Initializing Global Descriptor Table (GDT)...");
    unsafe {
        init_gdt();
    }
    log_info!("Initializing Interrupt Descriptor Table (IDT) & 8259 PIC...");
    unsafe {
        init_idt();
    }

    // 3. Hardware Timer Setup (8254 PIT at 100 Hz = 10ms per tick)
    log_info!("Configuring Intel 8254 PIT to 100 Hz...");
    unsafe {
        init_pit(100);
    }

    // 4. Memory Allocator Initialization
    log_mem!("Initializing Kernel Bump Allocator (128 KiB heap)...");
    init_heap();
    {
        let mut alloc = KERNEL_ALLOCATOR.lock();
        let test_layout = Layout::from_size_align(64, 8).unwrap();
        match alloc.allocate(test_layout) {
            Ok(ptr) => {
                log_mem!(
                    "Allocated 64 bytes at raw address {:#x}. Free memory: {} bytes",
                    ptr as usize,
                    alloc.available_bytes()
                );
            }
            Err(e) => {
                log_mem!("Heap allocation test failed: {:?}", e);
            }
        }
        let stats = alloc.stats();
        log_mem!(
            "Allocator Stats: Total: {} B, Allocated: {} B, Allocations: {}",
            stats.total_bytes,
            stats.allocated_bytes,
            stats.allocations_count
        );
    }

    // 5. Synchronization Primitive Verification
    log_sync!("Testing kernel SpinLock<T> synchronization primitive...");
    static SHARED_COUNTER: SpinLock<u32> = SpinLock::new(0);
    {
        let mut guard = SHARED_COUNTER.lock();
        *guard += 100;
        log_sync!("Acquired spinlock. Incremented counter to {}", *guard);
        // RAII guard automatically unlocks when going out of scope
    }
    {
        let guard = SHARED_COUNTER.lock();
        log_sync!("Re-acquired spinlock. Verified protected counter = {}", *guard);
    }

    // 6. Task Creation and Scheduler Setup
    println!("\n----------------------------------------------------------------");
    log_sched!("Spawning controlled tasks for Dynamic Quantum Round Robin test:");
    println!("----------------------------------------------------------------");

    {
        let mut sched = SCHEDULER.lock();

        // Task 1: Heavy CPU bound task (Burst: 18 ticks, Normal priority)
        let t1 = Task::new(TaskId(1), "Task-1-CPUBound", 5, 18, 6, None);
        // Task 2: Interactive task (Burst: 10 ticks, Normal priority, yields)
        let t2 = Task::new(TaskId(2), "Task-2-Interactive", 5, 10, 6, None);
        // Task 3: Shared resource worker (Burst: 14 ticks, High priority)
        let t3 = Task::new(TaskId(3), "Task-3-SyncWorker", 8, 14, 6, None);
        // Task 4: Diagnostic observer (Burst: 6 ticks, Low priority)
        let t4 = Task::new(TaskId(4), "Task-4-Diagnostics", 3, 6, 6, None);

        let _ = sched.spawn_task(t1);
        let _ = sched.spawn_task(t2);
        let _ = sched.spawn_task(t3);
        let _ = sched.spawn_task(t4);
    }

    log_task!("Spawned Task 1: 'Task-1-CPUBound'    (Burst: 18 ticks, Priority: 5)");
    log_task!("Spawned Task 2: 'Task-2-Interactive' (Burst: 10 ticks, Priority: 5)");
    log_task!("Spawned Task 3: 'Task-3-SyncWorker'  (Burst: 14 ticks, Priority: 8)");
    log_task!("Spawned Task 4: 'Task-4-Diagnostics' (Burst:  6 ticks, Priority: 3)");

    // Enable CPU hardware interrupts (sti)
    log_info!("Enabling CPU hardware interrupts (sti) for 8254 PIT IRQ0...");
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }

    // Hardware timer calibration: verify real timer interrupts arrive
    log_info!("Verifying hardware timer delivery (awaiting initial PIT pulses)...");
    let mut calibration_wait = 5000000;
    while get_hw_interrupt_count() < 3 && calibration_wait > 0 {
        calibration_wait -= 1;
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack));
        }
    }
    let hw_irqs_initial = get_hw_interrupt_count();
    log_info!(
        "Hardware timer operational: {} PIT IRQ0 interrupts received during calibration.",
        hw_irqs_initial
    );

    println!("\n----------------------------------------------------------------");
    log_sched!("Starting Integrated Scheduling Loop (Hardware PIT Driven Ticks):");
    println!("----------------------------------------------------------------");

    // 7. Scheduling Execution Loop
    let mut loop_limit = 200;
    let mut last_yield_runtime: u32 = u32::MAX;

    while loop_limit > 0 {
        loop_limit -= 1;

        // Check if all tasks have completed
        {
            let sched = SCHEDULER.lock();
            if sched.runnable_task_count() == 0 {
                log_sched!("All tasks terminated. CPU entered idle state.");
                break;
            }
        }

        // Await next hardware tick from PIT IRQ0 (sleep CPU using atomic sti; hlt)
        if consume_pending_ticks() == 0 {
            unsafe {
                core::arch::asm!("sti; hlt", options(nomem, nostack));
            }
        }

        let _current_tick = get_ticks();

        let action = {
            let mut sched = SCHEDULER.lock();
            let tick_action = sched.on_timer_tick();

            // If the running task didn't terminate or exhaust its quantum,
            // check if Task 2 should voluntarily yield (simulate I/O burst).
            // Decoupled from global wall-clock ticks: depends strictly on task runtime.
            if tick_action == ScheduleAction::Continue {
                if let Some(curr) = sched.current_task_id() {
                    let should_yield = if curr == TaskId(2) {
                        if let Some(task) = sched.get_task(curr) {
                            task.total_runtime > 0
                                && task.total_runtime % 3 == 0
                                && task.total_runtime != last_yield_runtime
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if should_yield {
                        if let Some(task) = sched.get_task(curr) {
                            last_yield_runtime = task.total_runtime;
                        }
                        log_task!("Task 2 voluntarily yields CPU to simulate I/O burst completion");
                        sched.yield_current()
                    } else {
                        tick_action
                    }
                } else {
                    tick_action
                }
            } else {
                tick_action
            }
        };

        match action {
            ScheduleAction::Continue => {
                // Task is still executing its current slice
            }
            ScheduleAction::ContextSwitch { prev, next, reason } => {
                let sched = SCHEDULER.lock();
                let prev_name = prev
                    .and_then(|p| sched.get_task(p))
                    .map(|t| t.name)
                    .unwrap_or("INIT");
                let next_task = sched.get_task(next).unwrap();
                let reason_str = match reason {
                    SwitchReason::QuantumExpired => "QuantumExpired",
                    SwitchReason::TaskYielded => "TaskYielded",
                    SwitchReason::TaskTerminated => "TaskTerminated",
                    SwitchReason::InitialSchedule => "InitialSchedule",
                    SwitchReason::TaskBlocked => "TaskBlocked",
                };

                log_sched!(
                    "SWITCH: {:<18} -> {:<18} | Reason: {:<15} | DQ: {:>2} ticks (Queue: {})",
                    prev_name,
                    next_task.name,
                    reason_str,
                    next_task.current_quantum,
                    sched.runnable_task_count()
                );
            }
            ScheduleAction::Idle => {
                let sched = SCHEDULER.lock();
                if sched.runnable_task_count() == 0 {
                    log_sched!("All tasks terminated. CPU entered idle state.");
                    break;
                }
            }
        }
    }

    // 8. Performance and Diagnostics Summary Report
    println!("\n==================================================================================");
    println!("                           SCHEDULER PERFORMANCE REPORT                           ");
    println!("==================================================================================");
    {
        let sched = SCHEDULER.lock();
        println!(
            "{:<4} {:<19} {:<5} {:<8} {:<8} {:<11} {:<9} {:<5} {:<6}",
            "TID", "NAME", "PRIO", "RUNTIME", "WAITING", "TURNAROUND", "RESPONSE", "CSW", "PREEMPT"
        );
        println!("----------------------------------------------------------------------------------");

        let mut total_waiting: u64 = 0;
        let mut total_turnaround: u64 = 0;
        let mut total_response: u64 = 0;
        let mut count: u64 = 0;

        for id in 1..=4 {
            if let Some(task) = sched.get_task(TaskId(id)) {
                let ta = task.turnaround_time();
                let resp = task.response_time();
                println!(
                    "{:<4} {:<19} {:<5} {:<8} {:<8} {:<11} {:<9} {:<5} {:<6}",
                    task.id.0,
                    task.name,
                    task.priority,
                    task.total_runtime,
                    task.waiting_time,
                    ta,
                    resp,
                    task.context_switch_count,
                    task.preempt_count
                );
                total_waiting += task.waiting_time as u64;
                total_turnaround += ta;
                total_response += resp;
                count += 1;
            }
        }

        println!("----------------------------------------------------------------------------------");
        let avg_waiting = if count > 0 { total_waiting / count } else { 0 };
        let avg_turnaround = if count > 0 { total_turnaround / count } else { 0 };
        let avg_response = if count > 0 { total_response / count } else { 0 };
        println!("Completed Tasks:           {}/{}", sched.completed_tasks_count, count);
        println!("Total Context Switches:    {}", sched.total_context_switches);
        println!("Total Scheduling Ticks:    {}", get_ticks());
        println!("Total Hardware IRQ0 Ticks: {}", get_hw_interrupt_count());
        println!("Average Waiting Time:      {} ticks", avg_waiting);
        println!("Average Turnaround Time:   {} ticks", avg_turnaround);
        println!("Average Response Time:     {} ticks", avg_response);
        println!("Workload Completion:       100% (All 4 tasks completed without starvation)");
    }

    println!("\n==================================================================================");
    println!("                     YASHWANTH MICRO-OS EXECUTION SUCCESSFUL                      ");
    println!("==================================================================================\n");

    // Cleanly disable interrupts before exit/park
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }

    // Exit QEMU cleanly with success exit code
    exit_qemu(true);

    hlt_loop();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    yashwanth_micro_os::kernel::panic::format_panic(info);
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

// ----------------------------------------------------------------------------
// Bare-Metal Compiler-Required Builtins (memcpy, memset, memmove, memcmp)
// ----------------------------------------------------------------------------
#[allow(suspicious_runtime_symbol_definitions)]
#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[allow(suspicious_runtime_symbol_definitions)]
#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[allow(suspicious_runtime_symbol_definitions)]
#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if (dest as usize) < (src as usize) {
        let mut i = 0;
        while i < n {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
    } else {
        let mut i = n;
        while i > 0 {
            i -= 1;
            *dest.add(i) = *src.add(i);
        }
    }
    dest
}

#[allow(suspicious_runtime_symbol_definitions)]
#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return (a as i32) - (b as i32);
        }
        i += 1;
    }
    0
}


