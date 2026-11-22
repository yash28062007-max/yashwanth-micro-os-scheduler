//! Simple Kernel Memory Management: Bump (Linear) Allocator
//!
//! Allocates memory linearly from a contiguous memory block.
//! Fast, deterministic O(1) allocation suitable for kernel initialization,
//! task stack allocation, and diagnostic buffers.

use core::alloc::Layout;
use crate::sync::SpinLock;

/// Allocation error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    InvalidAlignment,
    ZeroSizedAllocation,
}

/// Statistics for memory subsystem observability.
#[derive(Debug, Clone, Copy, Default)]
pub struct AllocatorStats {
    pub total_bytes: usize,
    pub allocated_bytes: usize,
    pub allocations_count: usize,
    pub peak_allocated_bytes: usize,
    pub failed_allocations: usize,
}

/// A thread-safe, deterministic bump allocator for educational Micro-OS.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    stats: AllocatorStats,
}

impl BumpAllocator {
    /// Creates an uninitialized bump allocator.
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            stats: AllocatorStats {
                total_bytes: 0,
                allocated_bytes: 0,
                allocations_count: 0,
                peak_allocated_bytes: 0,
                failed_allocations: 0,
            },
        }
    }

    /// Initializes the allocator with a raw memory region [heap_start, heap_start + size).
    ///
    /// # Safety
    /// The caller must ensure the memory region is valid, mapped, and not used elsewhere.
    pub unsafe fn init(&mut self, heap_start: usize, size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + size;
        self.next = heap_start;
        self.stats.total_bytes = size;
        self.stats.allocated_bytes = 0;
        self.stats.allocations_count = 0;
        self.stats.peak_allocated_bytes = 0;
        self.stats.failed_allocations = 0;
    }

    /// Allocates a block of memory with the given size and alignment.
    pub fn allocate(&mut self, layout: Layout) -> Result<*mut u8, AllocError> {
        let size = layout.size();
        let align = layout.align();

        if size == 0 {
            return Err(AllocError::ZeroSizedAllocation);
        }
        if !align.is_power_of_two() {
            return Err(AllocError::InvalidAlignment);
        }

        let alloc_start = align_up(self.next, align);
        let alloc_end = match alloc_start.checked_add(size) {
            Some(end) => end,
            None => {
                self.stats.failed_allocations += 1;
                return Err(AllocError::OutOfMemory);
            }
        };

        if alloc_end > self.heap_end {
            self.stats.failed_allocations += 1;
            return Err(AllocError::OutOfMemory);
        }

        self.next = alloc_end;
        self.stats.allocated_bytes = self.next - self.heap_start;
        self.stats.allocations_count += 1;
        if self.stats.allocated_bytes > self.stats.peak_allocated_bytes {
            self.stats.peak_allocated_bytes = self.stats.allocated_bytes;
        }

        Ok(alloc_start as *mut u8)
    }

    /// Resets the bump pointer to the beginning of the heap.
    ///
    /// # Safety
    /// All previously allocated references become invalid and must not be accessed.
    pub unsafe fn reset(&mut self) {
        self.next = self.heap_start;
        self.stats.allocated_bytes = 0;
    }

    /// Returns a snapshot of allocator statistics.
    pub fn stats(&self) -> AllocatorStats {
        self.stats
    }

    /// Returns remaining available memory in bytes.
    pub fn available_bytes(&self) -> usize {
        if self.next <= self.heap_end {
            self.heap_end - self.next
        } else {
            0
        }
    }
}

/// Helper function to align an address upwards to the nearest multiple of `align`.
#[inline]
pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Global kernel heap memory (128 KiB educational static heap).
const KERNEL_HEAP_SIZE: usize = 128 * 1024;
static mut KERNEL_HEAP_MEMORY: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// Global kernel bump allocator protected by a SpinLock.
pub static KERNEL_ALLOCATOR: SpinLock<BumpAllocator> = SpinLock::new(BumpAllocator::new());

/// Initializes the global kernel bump allocator.
pub fn init_heap() {
    let mut allocator = KERNEL_ALLOCATOR.lock();
    unsafe {
        let heap_ptr = KERNEL_HEAP_MEMORY.as_mut_ptr() as usize;
        allocator.init(heap_ptr, KERNEL_HEAP_SIZE);
    }
}
