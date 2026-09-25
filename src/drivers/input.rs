use core::pin::Pin;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll};

use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use crossbeam_queue::ArrayQueue;
use futures_util::Stream;
use futures_util::task::AtomicWaker;

use crate::common::mutex::debug_mutex::Mutex;

pub mod keyboard;

pub fn init() {
    keyboard::init();
}

pub struct SubscriberInner<T> {
    queue: ArrayQueue<T>,
    waker: AtomicWaker,
    dropped: AtomicU32,
}

const SUBSCRIBER_BUFFER_SIZE: usize = 32;

impl<T> SubscriberInner<T> {
    fn new() -> Self {
        Self {
            queue: ArrayQueue::new(SUBSCRIBER_BUFFER_SIZE),
            waker: AtomicWaker::new(),
            dropped: AtomicU32::new(0),
        }
    }

    fn push(&self, event: T) {
        if self.queue.push(event).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
        self.waker.wake();
    }

    pub fn take_dropped(&self) -> u32 {
        self.dropped.swap(0, Ordering::Relaxed)
    }

    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    pub fn poll_next(&self, cx: &mut Context<'_>) -> Poll<T> {
        if let Some(e) = self.queue.pop() { return Poll::Ready(e); }
        self.waker.register(cx.waker());
        match self.queue.pop() {
            Some(e) => { self.waker.take(); Poll::Ready(e) }
            None => Poll::Pending,
        }
    }
}

pub struct Reader<T>(pub Arc<SubscriberInner<T>>);

impl<T> Stream for Reader<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.poll_next(cx).map(Some)
    }
}

pub struct InputHub<T> {
    subs: Mutex<Vec<Weak<SubscriberInner<T>>>>,
}

impl<T> InputHub<T> {
    pub const fn new() -> Self {
        Self { subs: Mutex::new(Vec::new()) }
    }
}

impl<T> Default for InputHub<T> {
    fn default() -> Self {
        Self { subs: Mutex::new(Vec::new()) }
    }
}

impl<T: Clone + Send + Sync> InputHub<T> {
    pub fn subscribe(&self) -> Reader<T> {
        let sub = Arc::new(SubscriberInner::new());
        self.subs.lock().push(Arc::downgrade(&sub));
        Reader(sub)
    }

    fn push(&self, event: T) {
        let mut guard = self.subs.lock();
        guard.retain(|sub| {
            if let Some(sub) = sub.upgrade() {
                sub.push(event.clone());
                true
            } else { false }
        });
    }
}

