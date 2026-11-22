//! Core Kernel Management Subsystem
//!
//! Provides boot protocol handling, structured logging, panic recovery,
//! descriptor tables, and CPU exception dispatching.

pub mod gdt;
pub mod idt;
pub mod limine_proto;
pub mod log;
pub mod panic;

pub use gdt::init_gdt;
pub use idt::{init_idt, send_eoi};
