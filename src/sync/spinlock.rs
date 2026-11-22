//! Basic Synchronization Mechanism: Kernel SpinLock
//!
//! Implements a mutual exclusion primitive using an atomic boolean flag (`AtomicBool`)
//! with busy-waiting (spinning) and an RAII lock guard (`SpinLockGuard`).
//! Provides interrupt-safe locking in bare-metal mode to prevent deadlocks
//! caused by interrupt handlers trying to acquire the same lock.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(any(target_os = "none", feature = "kernel_bin"))]
use core::arch::asm;

/// A simple, robust kernel spinlock.
pub struct SpinLock<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}
unsafe impl<T: ?Sized + Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Creates a new spinlock protecting the given data.
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> SpinLock<T> {
    /// Acquires the lock, spinning until it becomes available.
    ///
    /// In bare-metal target, interrupts are disabled during the spin to avoid
    /// deadlock if an interrupt handler attempts to acquire this lock.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        let saved_flags = Self::disable_interrupts();
        while self.lock.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            #[cfg(any(target_os = "none", feature = "kernel_bin"))]
            unsafe {
                asm!("pause", options(nomem, nostack, preserves_flags));
            }
            #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
            core::hint::spin_loop();
        }

        SpinLockGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
            saved_flags,
        }
    }

    /// Tries to acquire the lock without blocking.
    /// Returns `Some(SpinLockGuard)` if acquired, or `None` if already locked.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        let saved_flags = Self::disable_interrupts();
        if self.lock.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_ok() {
            Some(SpinLockGuard {
                lock: &self.lock,
                data: unsafe { &mut *self.data.get() },
                saved_flags,
            })
        } else {
            Self::restore_interrupts(saved_flags);
            None
        }
    }

    /// Checks if the lock is currently held.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    #[inline]
    fn disable_interrupts() -> u64 {
        #[cfg(any(target_os = "none", feature = "kernel_bin"))]
        unsafe {
            let flags: u64;
            asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags));
            asm!("cli", options(nomem, nostack));
            flags
        }
        #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
        0
    }

    #[inline]
    fn restore_interrupts(flags: u64) {
        #[cfg(any(target_os = "none", feature = "kernel_bin"))]
        unsafe {
            if (flags & (1 << 9)) != 0 {
                asm!("sti", options(nomem, nostack));
            }
        }
        #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
        let _ = flags;
    }
}

/// RAII guard returned by `SpinLock::lock()`.
/// Automatically releases the spinlock when dropped.
pub struct SpinLockGuard<'a, T: ?Sized> {
    lock: &'a AtomicBool,
    data: &'a mut T,
    saved_flags: u64,
}

impl<'a, T: ?Sized> Deref for SpinLockGuard<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinLockGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.store(false, Ordering::Release);
        SpinLock::<T>::restore_interrupts(self.saved_flags);
    }
}
