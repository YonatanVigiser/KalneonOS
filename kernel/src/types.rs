pub struct SimpleMutex<T> {
    data: core::cell::UnsafeCell<T>,
}

impl<T> SimpleMutex<T> {
    pub const fn new(data: T) -> Self {
        SimpleMutex { data: core::cell::UnsafeCell::new(data) }
    }

    pub fn lock(&self) -> &mut T {
        // SAFETY: single-threaded kernel, so no concurrency issues
        unsafe { &mut *self.data.get() }
    }
}

unsafe impl<T> Sync for SimpleMutex<T> {}
unsafe impl<T> Send for SimpleMutex<T> {}

use core::sync::atomic::{AtomicBool, Ordering};
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;

pub struct SimpleOnce<T> {
    initialized: AtomicBool,
    value: UnsafeCell<core::mem::MaybeUninit<T>>,
}

impl<T> SimpleOnce<T> {
    pub const fn new() -> Self {
        SimpleOnce {
            initialized: AtomicBool::new(false),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn call_once(&self, init: impl FnOnce() -> T) {
        if !self.initialized.load(Ordering::Acquire) {
            unsafe {
                (*self.value.get()).write(init());
            }
            self.initialized.store(true, Ordering::Release);
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.initialized.load(Ordering::Acquire) {
            Some(unsafe { &*(*self.value.get()).as_ptr() })
        } else {
            None
        }
    }
}

unsafe impl<T> Sync for SimpleOnce<T> {}

