//! Memory Management Subsystem
//!
//! Provides bare-metal memory allocation primitives and statistics.

pub mod bump;

pub use bump::{align_up, AllocError, AllocatorStats, BumpAllocator, init_heap, KERNEL_ALLOCATOR};
