pub mod executor;
pub mod scheduler;
pub mod sleep;
pub mod waker;

use alloc::{boxed::Box, sync::Arc};
use atomic_enum::atomic_enum;
#[cfg(debug_assertions)]
use core::sync::atomic::AtomicBool;
use core::{
    any::type_name_of_val, cell::UnsafeCell, future::Future, pin::Pin, sync::atomic::{AtomicU64, Ordering}, task::{Context, Poll}
};

use crate::{arch::cpu::current_cpu, task::executor::EXECUTOR, time::KernelDuration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)] pub struct TaskId(u64);

impl TaskId {
    pub const EMPTY: Self = Self(0);

    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn current() -> Option<Self> {
        current_cpu().current_task_id.get()
    }

    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

const DEFAULT_AFFINITY_THRESHOLD_RATIO: f64 = 3.0;
const EWMA_CONSTANT: f64 = 0.05;
const LONG_AVERAGE_RUNTIME: KernelDuration = KernelDuration::from_micros(500);

#[atomic_enum]
#[derive(PartialEq, Eq)]
pub enum TaskState {
    Idle,
    Scheduled,
    Running,
    Notified,
    Completed,
}

pub struct Task {
    id: TaskId,
    name: &'static str,
    state: AtomicTaskState,
    future: UnsafeCell<Pin<Box<dyn Future<Output = ()>>>>,
    pinned: bool,
    affinity_threshold: f64,
    average_runtime_nanos: AtomicU64,
    slow: AtomicBool,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            id: TaskId::new(),
            name: type_name_of_val(&future),
            state: AtomicTaskState::new(TaskState::Idle),
            future: UnsafeCell::new(Box::pin(future)),
            pinned: false,
            affinity_threshold: DEFAULT_AFFINITY_THRESHOLD_RATIO,
            average_runtime_nanos: AtomicU64::new(0),
            slow: AtomicBool::new(false)
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

    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    pub fn affinity_threshold(&self) -> f64 {
        self.affinity_threshold
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    fn average_runtime_nanos(&self) -> u64 {
        self.average_runtime_nanos.load(Ordering::Relaxed)
    }

    fn poll(&self, context: &mut Context) -> Poll<()> {
        (unsafe { self.future.as_mut_unchecked() })
            .as_mut()
            .poll(context)
    }

    fn record_runtime(&self, runtime: KernelDuration) {
        let old_time = self.average_runtime_nanos.load(Ordering::Relaxed) as f64;
        let new_avg = (old_time * (1.0 - EWMA_CONSTANT) + runtime.as_nanos() as f64 * EWMA_CONSTANT) as u64;
        self.average_runtime_nanos.store(new_avg, Ordering::Relaxed);
        let slow = new_avg > LONG_AVERAGE_RUNTIME.as_nanos();
        if self.slow.swap(slow, Ordering::Relaxed) != slow && new_avg > LONG_AVERAGE_RUNTIME.as_nanos() {
            log::warn!("Long task poll average {}: {}", self.name, new_avg);
        }
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
