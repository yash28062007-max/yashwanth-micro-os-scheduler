//! Structured Kernel Logging and Diagnostics
//!
//! Provides tagged, formatted serial logging for observability across
//! scheduler transitions, task lifecycle, synchronization, and memory events.

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::println!("[TICKS:{:06}] [INFO ] {}", $crate::timer::get_ticks(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::println!("[TICKS:{:06}] [WARN ] {}", $crate::timer::get_ticks(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::println!("[TICKS:{:06}] [ERROR] {}", $crate::timer::get_ticks(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_sched {
    ($($arg:tt)*) => {
        $crate::println!("[TICKS:{:06}] [SCHED] {}", $crate::timer::get_ticks(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_task {
    ($($arg:tt)*) => {
        $crate::println!("[TICKS:{:06}] [TASK ] {}", $crate::timer::get_ticks(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_sync {
    ($($arg:tt)*) => {
        $crate::println!("[TICKS:{:06}] [SYNC ] {}", $crate::timer::get_ticks(), format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_mem {
    ($($arg:tt)*) => {
        $crate::println!("[TICKS:{:06}] [MEM  ] {}", $crate::timer::get_ticks(), format_args!($($arg)*))
    };
}
