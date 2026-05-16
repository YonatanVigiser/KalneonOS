use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::time::{timer::Timer, uptime_nano};


#[must_use]
pub struct Sleep {
    deadline_nanos: u64,
    registered: bool,
}

impl Sleep {
    pub fn new(deadline_nanos: u64) -> Self {
        Self {
            deadline_nanos,
            registered: false,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if uptime_nano() >= self.deadline_nanos {
            return Poll::Ready(());
        }
        if !self.registered {
            Timer::new(self.deadline_nanos, cx.waker().clone()).register();
            self.as_mut().registered = true;
        }
        Poll::Pending
    }
}

pub fn sleep(time_nanos: u64) -> Sleep {
    Sleep::new(time_nanos + uptime_nano())
}
