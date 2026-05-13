pub mod executor;
pub mod scheduler;
pub mod sleep;

use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: TaskId,
    future: UnsafeCell<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: TaskId::new(),
            future: UnsafeCell::new(Box::pin(future)),
        }
    }

    pub fn poll(&self, context: &mut Context) -> Poll<()> {
        (unsafe { self.future.as_mut_unchecked() })
            .as_mut()
            .poll(context)
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

pub struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn yield_now() -> YieldNow {
    YieldNow(false)
}
