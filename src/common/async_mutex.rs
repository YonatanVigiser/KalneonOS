use core::{cell::UnsafeCell, ops::{Deref, DerefMut}, pin::Pin, sync::atomic::{AtomicBool, AtomicU64, Ordering}, task::{Context, Poll, Waker}};

use crossbeam_queue::SegQueue;

use crate::task::{Task, TaskId};

pub struct AsyncMutex<T> {
    holder: AtomicU64,
    locked: AtomicBool,
    waiters: SegQueue<Waker>,
    data: UnsafeCell<T>,
}

impl<T> AsyncMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            holder: AtomicU64::new(TaskId::EMPTY.as_u64()),
            locked: AtomicBool::new(false),
            waiters: SegQueue::new(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn try_lock(&self) -> Option<AsyncMutexGuard<'_, T>> {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
            .then(|| {
                self.holder.store(TaskId::current().unwrap_or(TaskId::EMPTY).as_u64(), Ordering::Release);
                AsyncMutexGuard { mutex: self }
            })
    }

    pub fn lock(&self) -> LockFuture<'_, T> {
        LockFuture { mutex: self }
    }

    pub fn lock_blocking(&self) -> AsyncMutexGuard<'_, T> {
        debug_assert!(Task::current().is_none(), "lock_blocking was called from inside a task!");
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            }
            core::hint::spin_loop();
        }
    }
}

unsafe impl<T: Send> Send for AsyncMutex<T> {}
unsafe impl<T: Send> Sync for AsyncMutex<T> {}

pub struct LockFuture<'a, T> {
    mutex: &'a AsyncMutex<T>,
}

pub struct AsyncMutexGuard<'a, T> {
    mutex: &'a AsyncMutex<T>,
}

impl<T> Deref for AsyncMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for AsyncMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for AsyncMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
        self.mutex.holder.store(TaskId::EMPTY.as_u64(), Ordering::Release);
        if let Some(waker) = self.mutex.waiters.pop() {
            waker.wake();
        }
    }
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = AsyncMutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Some(guard) = self.mutex.try_lock() {
            return Poll::Ready(guard);
        }
        self.mutex.waiters.push(cx.waker().clone());
        // Re-check to avoid race conditions
        if let Some(guard) = self.mutex.try_lock() {
            return Poll::Ready(guard);
        }
        Poll::Pending
    }
}
