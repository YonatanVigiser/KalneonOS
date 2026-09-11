use core::future::poll_fn;
use core::ops::Deref;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use core::task::{Context, Poll, Waker};

use alloc::sync::Arc;
use alloc::vec::Vec;
use atomic_enum::atomic_enum;
use futures_util::task::AtomicWaker;
use spin::Mutex;

pub struct Shared;
pub struct Exclusive;

pub trait Access: 'static { const EXLUSIVE: bool; }
impl Access for Shared    { const EXLUSIVE: bool = false; }
impl Access for Exclusive { const EXLUSIVE: bool = true; }

#[atomic_enum]
pub enum LeaseStatus {
    Free,
    Owned,
    ReleaseRequested,
}

pub struct LeaseState {
    status: AtomicLeaseStatus,
    owner_waker: AtomicWaker,
    waiters: Mutex<Vec<Waker>>
}

impl Default for LeaseState {
    fn default() -> Self {
        Self {
            status: AtomicLeaseStatus::new(LeaseStatus::Free),
            owner_waker: AtomicWaker::new(),
            waiters: Mutex::new(Vec::new())
        }
    }
}

impl LeaseState {
    pub(super) fn try_acquire(&self) -> bool {
        self.status.compare_exchange(LeaseStatus::Free, LeaseStatus::Owned, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    pub(super) fn release(&self) {
        self.owner_waker.take();
        self.status.store(LeaseStatus::Free, Ordering::Release);
        for waker in self.waiters.lock().drain(..) {
            waker.wake();
        }
    }

    pub(super) fn register_waiter(&self, waker: &Waker) {
        let mut waiters = self.waiters.lock();
        if !waiters.iter().any(|w| w.will_wake(waker)) {
            waiters.push(waker.clone());
        }
    }

    pub(super) fn request_release(&self) {
        if self.status.compare_exchange(LeaseStatus::Owned, LeaseStatus::ReleaseRequested, Ordering::AcqRel, Ordering::Acquire).is_ok() {
            self.owner_waker.wake();
        }
    }

    pub fn status(&self) -> LeaseStatus {
        self.status.load(Ordering::Acquire)
    }

    pub fn release_requested(&self) -> bool {
        matches!(self.status.load(Ordering::Acquire), LeaseStatus::ReleaseRequested)
    }

    pub fn poll_release_requested(&self, cx:  &mut Context<'_>) -> Poll<()> {
        if self.release_requested() {
            return Poll::Ready(());
        }
        self.owner_waker.register(cx.waker());
        if self.release_requested() {
            Poll::Ready(())
        } else { Poll::Pending }
    }

    pub async fn poll_release_request(&self) {
        poll_fn(|cx| self.poll_release_requested(cx)).await
    }
}

pub struct Lease<R: ?Sized> {
    dev: Arc<R>,
    state: Arc<LeaseState>,
}

impl<R: ?Sized> Lease<R> {
    pub(super) fn new(dev: Arc<R>, state: Arc<LeaseState>) -> Self {
        Self { dev, state }
    }

    pub fn state(&self) -> Arc<LeaseState> {
        self.state.clone()
    }
}

impl<R: ?Sized> Deref for Lease<R> {
    type Target = R;
    fn deref(&self) -> &R { &self.dev }
}

impl<R: ?Sized> Drop for Lease<R> {
    fn drop(&mut self) { self.state.release(); }
}

pub struct Acquire<R: ?Sized> {
    dev: Arc<R>,
    state: Arc<LeaseState>,
}

impl<R: ?Sized> Acquire<R> {
    pub(super) fn new(dev: Arc<R>, state: Arc<LeaseState>) -> Self {
        Self { dev, state }
    }
}

impl<R: ?Sized> Future for Acquire<R> {
    type Output = Lease<R>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Lease<R>> {
        if self.state.try_acquire() {
            return Poll::Ready(Lease::new(self.dev.clone(), self.state.clone()));
        }
        self.state.register_waiter(cx.waker());
        if self.state.try_acquire() {
            return Poll::Ready(Lease::new(self.dev.clone(), self.state.clone()));
        }
        Poll::Pending
    }
}
