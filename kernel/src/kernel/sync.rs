use core::cell::UnsafeCell;
use crate::kernel::threading::scheduler;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Shared<T: ?Sized> {
    locked: AtomicBool,
    cell: UnsafeCell<T>,
}

pub struct SharedGuard<'a, T: ?Sized> {
    data: *mut T,
    lock: &'a AtomicBool,
}

unsafe impl<T: ?Sized + Send> Sync for Shared<T> {}
unsafe impl<T: ?Sized + Send> Send for Shared<T> {}

unsafe impl<T: ?Sized + Sync> Sync for SharedGuard<'_, T> {}
unsafe impl<T: ?Sized + Send> Send for SharedGuard<'_, T> {}

impl<T> Shared<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            cell: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SharedGuard<'_, T> {
        while self.locked.swap(true, Ordering::Acquire) {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
        scheduler::disable_preemption();
        SharedGuard {
            data: self.cell.get(),
            lock: &self.locked,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }

    pub unsafe fn force_unlock(&self) {
        if self.locked.swap(false, Ordering::AcqRel) {
            scheduler::enable_preemption();
        }
    }
}

impl<T: ?Sized> Deref for SharedGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> DerefMut for SharedGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T: ?Sized> Drop for SharedGuard<'_, T> {
    fn drop(&mut self) {
        if self.lock.swap(false, Ordering::AcqRel) {
            scheduler::enable_preemption();
            if scheduler::preemption_enabled() && scheduler::SWITCH_MISSED.swap(false, Ordering::AcqRel) {
                scheduler::SCHEDULER.lock().yield_now();
            }
        }
    }
}
