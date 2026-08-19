use core::{cmp::Reverse, task::Waker};

use alloc::collections::binary_heap::BinaryHeap;
use spin::Mutex;

use crate::time::{KernelDuration, KernelInstant, uptime};

static TIMERS: Mutex<BinaryHeap<Reverse<Timer>>> = Mutex::new(BinaryHeap::new());


#[derive(Debug)]
pub struct Timer {
    start: KernelInstant,
    duration: KernelDuration,
    deadline: KernelInstant,
    waker: Waker,
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
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
        self.deadline.cmp(&other.deadline)
    }
}

impl Timer {
    pub fn new(duration: KernelDuration, waker: Waker) -> Self {
        let start = uptime();
        let deadline = start.checked_add_duration(duration).expect("Overflow");
        Self {
            start,
            duration,
            deadline,
            waker,
        }
    }

    pub fn expired(&self) -> bool {
        uptime() >= self.deadline
    }

    pub fn register(self) {
        TIMERS.lock().push(Reverse(self));
    }

    pub async fn wake_timers() {
        loop {
            let mut timers = TIMERS.lock();
            while let Some(&Reverse(timer)) = timers.peek().as_ref() {
                if !timer.expired() { break; }
                let timer = timers.pop().expect("Shouldn't fail");
                timer.0.waker.wake_by_ref();
            }
            crate::task::yield_now().await;
        }
    }
}
