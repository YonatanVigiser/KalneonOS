use core::sync::atomic::{AtomicUsize, Ordering};

use lock_api::{GuardNoSend, RawMutex};
use spin::mutex::SpinMutex;

use crate::arch::cpu::{CpuId, current_cpu};
use crate::time::{KernelDuration, uptime};

pub struct RawDebugMutex<R: RawMutex> {
    inner: R,
    holder: AtomicUsize,
}

const NO_HOLDER: usize = usize::MAX;
const MAX_LOCK_DURATION: KernelDuration = KernelDuration::from_millis(100);
const UPTIME_CHECK_ITERATIONS: usize = 10_000;

impl<R: RawMutex> RawDebugMutex<R> {
    pub const fn new() -> Self {
        Self {
            inner: R::INIT,
            holder: AtomicUsize::new(NO_HOLDER),
        }
    }
}

unsafe impl<R: RawMutex> RawMutex for RawDebugMutex<R> {
    const INIT: Self = RawDebugMutex::new();
    type GuardMarker = GuardNoSend;

    fn lock(&self) {
        let current_id = current_cpu().logical_id;
        if self.holder.load(Ordering::Relaxed) == current_id.0 {
            panic!("{} tried locking the same Mutex twice! Deadlock!", current_id);
        }
        let start_uptime = uptime();
        let mut count = 0;
        while !self.try_lock() {
            while self.inner.is_locked() {
                count += 1;
                if count % UPTIME_CHECK_ITERATIONS == 0 {
                    debug_assert!(uptime() - start_uptime < MAX_LOCK_DURATION, "Mutex Deadlock! Holder: {}, Deadlocked: {}",
                        CpuId(self.holder.load(Ordering::Relaxed)), current_id);
                }
                core::hint::spin_loop();
            }
        }
    }

    fn try_lock(&self) -> bool {
        if self.inner.try_lock() {
            self.holder.store(current_cpu().logical_id.0, Ordering::Relaxed);
            true
        } else { false }
    }

    unsafe fn unlock(&self) {
        self.holder.store(NO_HOLDER, Ordering::Relaxed);
        unsafe { self.inner.unlock(); }
    }

    fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
}

pub type Mutex<T> = lock_api::Mutex<RawDebugMutex<SpinMutex<()>>, T>;
pub type MutexGuard<'a, T> =
    lock_api::MutexGuard<'a, RawDebugMutex<SpinMutex<()>>, T>;
