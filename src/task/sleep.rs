use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::platform::uptime_nano;


pub struct Sleep {
    deadline: u64,
    registered: bool,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if uptime_nano() >= self.deadline {
            return Poll::Ready(());
        }
        if !self.registered {
            //register_timer(self.deadline, cx.waker().clone());
            self.as_mut().registered = true;
        }
        Poll::Pending
    }
}
