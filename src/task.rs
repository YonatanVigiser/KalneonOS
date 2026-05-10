pub mod executer;

use core::{future::Future, pin::Pin, sync::atomic::{AtomicU64, Ordering}, task::{Poll, Context}, cell::UnsafeCell};
use alloc::boxed::Box;

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
        (unsafe { self.future.as_mut_unchecked() }).as_mut().poll(context)
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

