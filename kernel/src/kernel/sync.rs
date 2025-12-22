use core::cell::UnsafeCell;
use crate::kernel::threading::scheduler;
use core::ops::{Deref, DerefMut};

pub struct Shared<T: ?Sized> {
    cell: UnsafeCell<T>,
}

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
