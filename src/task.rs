pub mod executor;
pub mod scheduler;
pub mod sleep;
pub mod waker;

use alloc::{boxed::Box, sync::Arc};
use atomic_enum::atomic_enum;
use core::{
    cell::UnsafeCell,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

use crate::{arch::cpu::current_cpu, task::executor::EXECUTOR};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TaskId(u64);

impl TaskId {
    pub const EMPTY: Self = Self(0);

    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn current() -> Option<Self> {
        Some(current_cpu().current_task_id?)
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

const DEFAULT_AFFINITY_THRESHOLD_RATIO: f64 = 3.0;

#[atomic_enum]
pub enum TaskState {
    Idle,
    Scheduled,
    Running,
    Notified,
    Completed,
}

pub struct Task {
    id: TaskId,
    state: AtomicTaskState,
    future: UnsafeCell<Pin<Box<dyn Future<Output = ()>>>>,
    pinned: bool,
    affinity_threshold: f64,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            id: TaskId::new(),
            state: AtomicTaskState::new(TaskState::Idle),
            future: UnsafeCell::new(Box::pin(future)),
            pinned: false,
            affinity_threshold: DEFAULT_AFFINITY_THRESHOLD_RATIO,
        }
    }

    pub fn current() -> Option<Arc<Self>> {
        Some(EXECUTOR.get()?.get_task(TaskId::current()?)?.clone())
    }

    pub fn pin(mut self) -> Self {
        self.pinned = true;
        self
    }

    pub fn with_affinity_threshold(mut self, affinity_threshold: f64) -> Self {
        self.affinity_threshold = affinity_threshold;
        self
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    pub fn affinity_threshold(&self) -> f64 {
        self.affinity_threshold
    }

    fn poll(&self, context: &mut Context) -> Poll<()> {
        (unsafe { self.future.as_mut_unchecked() })
            .as_mut()
            .poll(context)
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

#[must_use]
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
