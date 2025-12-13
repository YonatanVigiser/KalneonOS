use core::cell::UnsafeCell;
use crate::kernel::threading::scheduler;
use core::ops::{Deref, DerefMut};

/// Thread-safe shared memory accessor for single-core systems.
///
/// Provides mutual exclusion by disabling preemption during lock acquisition.
/// This prevents context switches while the lock is held, guaranteeing exclusive access.
///
/// # Safety guarantees
///
/// On single-core systems, disabling preemption is sufficient for mutual exclusion since:
/// - No other thread can be scheduled to run
/// - Interrupts can still occur but won't cause context switches
///
/// # Limitations
///
/// - NOT suitable for multi-core systems (requires actual atomic synchronization)
/// - If a panic occurs while holding the lock, preemption remains disabled
/// - Holding locks for extended periods degrades system responsiveness
pub struct Shared<T: ?Sized> {
    cell: UnsafeCell<T>,
}

/// RAII guard that maintains exclusive access while held.
///
/// Preemption is disabled for the lifetime of this guard and automatically
/// re-enabled when the guard is dropped.
pub struct SharedGuard<T: ?Sized> {
    data: *mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Shared<T> {}
unsafe impl<T: ?Sized + Send> Send for Shared<T> {}

unsafe impl<T: ?Sized + Sync> Sync for SharedGuard<T> {}
unsafe impl<T: ?Sized + Send> Send for SharedGuard<T> {}

impl<T> Shared<T> {
    pub const fn new(data: T) -> Self {
        Self {
            cell: UnsafeCell::new(data),
        }
    }

    /// Acquire exclusive access to the data.
    ///
    /// Disables preemption and returns a guard that provides mutable access.
    /// Preemption is re-enabled when the guard is dropped.
    ///
    /// # Safety
    ///
    /// Safe because:
    /// - Preemption is disabled before accessing the UnsafeCell
    /// - The returned pointer is uniquely owned by the guard
    /// - No context switch can occur until preemption is re-enabled
    pub fn lock(&self) -> SharedGuard<T> {
        scheduler::disable_preemption();
        SharedGuard {
            data: self.cell.get(),
        }
    }
}

impl<T: ?Sized> Deref for SharedGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> DerefMut for SharedGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T: ?Sized> Drop for SharedGuard<T> {
    fn drop(&mut self) {
        scheduler::enable_preemption();
    }
}
