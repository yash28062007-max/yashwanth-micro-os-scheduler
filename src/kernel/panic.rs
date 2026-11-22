//! Kernel Panic Handler and Diagnostic Formatter
//!
//! Captures fatal unrecoverable errors, logs diagnostic information to the
//! serial port, and places the CPU into a safe halted state.

use core::panic::PanicInfo;

pub fn format_panic(info: &PanicInfo) {
    crate::println!("\n========================================================");
    crate::println!("               !!! KERNEL PANIC !!!                    ");
    crate::println!("========================================================");

    if let Some(location) = info.location() {
        crate::println!("Location: {}:{}:{}", location.file(), location.line(), location.column());
    } else {
        crate::println!("Location: Unknown location");
    }

    crate::println!("Details:  {}", info);
    crate::println!("System Halted.");
    crate::println!("========================================================\n");
}
