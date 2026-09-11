use core::ops::Deref;
use core::pin::Pin;
use core::sync::atomic::Ordering;
use core::task::{Context, Poll, Waker};

use alloc::vec::Vec;
use alloc::sync::Arc;
use atomic_enum::atomic_enum;
use futures_util::task::AtomicWaker;
use spin::Mutex;

pub struct Shared;
pub struct Exclusive;

pub trait Access: 'static { const EXLUSIVE: bool; }
impl Access for Shared    { const EXLUSIVE: bool = false; }
impl Access for Exclusive { const EXLUSIVE: bool = true; }

#[atomic_enum]
enum LeaseStatus {
    Free,
    Owned,
    ReleaseRequested,
}

pub(super) struct LeaseState {
    status: AtomicLeaseStatus,
    owner_waker: AtomicWaker,
    waiters: Mutex<Vec<Waker>>
}

impl Default for LeaseState {
    fn default() -> Self {
        Self { status: AtomicLeaseStatus::new(LeaseStatus::Free), owner_waker: AtomicWaker::new(), waiters: Mutex::new(Vec::new()) }
    }
}

impl LeaseState {
    pub fn try_acquire(&self) -> bool {
        self.status.compare_exchange(LeaseStatus::Free, LeaseStatus::Owned, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    pub fn release(&self) {
    }

    pub fn register_waiter(&self, waker: &Waker) {
        let mut waiters = self.waiters.lock();
        if !waiters.iter().any(|w| w.will_wake(waker)) {
            waiters.push(waker.clone());
        }
    }


    pub fn request_release(&self) {
        if self.status.compare_exchange(LeaseStatus::Owned, LeaseStatus::ReleaseRequested, Ordering::AcqRel, Ordering::Acquire).is_ok() {
            self.owner_waker.wake();
        }
    }

    pub fn status(&self) -> LeaseStatus {
        self.status.load(Ordering::Acquire)
    }

    pub fn poll_release_requested(&self, cx:  &mut Context<'_>) -> Poll<()> {
        if 
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

    pub fn release_requested(&self) -> bool {
        self.state.release_requested.load(Ordering::Acquire)
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
    callback: Option<OnReleaseRequest>,
}

impl<R: ?Sized> Acquire<R> {
    pub(super) fn new(dev: Arc<R>, state: Arc<LeaseState>, callback: Option<OnReleaseRequest>) -> Self {
        Self { dev, state, callback }
    }
}

impl<R: ?Sized> Unpin for Acquire<R> {}

impl<R: ?Sized> Future for Acquire<R> {
    type Output = Lease<R>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Lease<R>> {
        if self.state.try_acquire(self.callback.clone()) {
            return Poll::Ready(Lease::new(self.dev.clone(), self.state.clone()));
        }
        self.state.register_waiter(cx.waker());
        if self.state.try_acquire(self.callback.clone()) {
            return Poll::Ready(Lease::new(self.dev.clone(), self.state.clone()));
        }
        Poll::Pending
    }
}
