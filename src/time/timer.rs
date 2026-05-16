use core::{cmp::Reverse, task::Waker};

use alloc::collections::binary_heap::BinaryHeap;
use spin::Mutex;

static TIMERS: Mutex<BinaryHeap<Reverse<Timer>>> = Mutex::new(BinaryHeap::new());

pub struct Timer {
    deadline_nanos: u64,
    waker: Waker,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.deadline_nanos == other.deadline_nanos
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deadline_nanos.cmp(&other.deadline_nanos)
    }
}

impl Timer {
    pub fn new(deadline_nanos: u64, waker: Waker) -> Self {
        Self {
            deadline_nanos,
            waker,
        }
    }

    pub fn register(self) {
        TIMERS.lock().push(Reverse(self));
    }

    pub async fn wake_timers() {
        loop {
            let mut timers = TIMERS.lock();
            while let Some(&Reverse(timer)) = timers.peek().as_ref() {
                if timer.deadline_nanos > super::uptime_nano() { break; }
                let timer = timers.pop().expect("Shouldn't fail");
                timer.0.waker.wake_by_ref();
            }
            crate::task::yield_now().await;
        }
    }
}
