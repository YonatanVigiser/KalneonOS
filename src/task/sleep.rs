use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::time::{KernelDuration, timer::Timer};


#[must_use]
pub struct Sleep {
    duration: KernelDuration,
    registered: bool,
}

impl Sleep {
    pub fn new(duration: KernelDuration) -> Self {
        Self {
            duration,
            registered: false,
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.registered {
            Timer::new(self.duration, cx.waker().clone()).register();
            self.as_mut().registered = true;
        }
        Poll::Pending
    }
}
