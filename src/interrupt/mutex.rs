use core::sync::atomic::{AtomicBool, Ordering};

use lock_api::{GuardNoSend, RawMutex};
use spin::mutex::SpinMutex;

use crate::interrupt;

pub struct RawInterruptSafeMutex<R: RawMutex> { inner: R, was_enabled: AtomicBool }

impl<R: RawMutex> RawInterruptSafeMutex<R> {
    pub const fn new() -> Self {
        Self { inner: R::INIT, was_enabled: AtomicBool::new(false) }
    }
}

unsafe impl<R: RawMutex> RawMutex for RawInterruptSafeMutex<R> {
    const INIT: Self = Self::new();
    type GuardMarker = GuardNoSend;

    fn lock(&self) {
        let was_enabled = interrupt::are_enabled();
        super::disable();
        self.inner.lock();
        self.was_enabled.store(was_enabled, Ordering::Relaxed);
    }

    fn try_lock(&self) -> bool {
        let was = super::are_enabled();
        super::disable();
        if self.inner.try_lock() {
            self.was_enabled.store(was, Ordering::Relaxed);
            true
        } else {
            if was { super::enable(); }
            false
        }
    }

    unsafe fn unlock(&self) {
        let was = self.was_enabled.load(Ordering::Relaxed);
        unsafe { self.inner.unlock() };
        if was { super::enable(); }
    }

    fn is_locked(&self) -> bool { self.inner.is_locked() }
}

pub type InterruptSafeMutex<T> = lock_api::Mutex<RawInterruptSafeMutex<SpinMutex<()>>, T>;
pub type InterruptSafeMutexGuard<'a, T> = lock_api::MutexGuard<'a, RawInterruptSafeMutex<SpinMutex<()>>, T>;
