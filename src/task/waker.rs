use core::task::Waker;

use alloc::{sync::{Arc, Weak}, task::Wake};

use crate::task::{Task, executor::EXECUTOR};

pub(super) struct TaskWaker {
    task: Weak<Task>,
    prev_core: usize,
}

impl TaskWaker {
    pub fn new_waker(task: Weak<Task>, prev_core: usize) -> Waker {
        Waker::from(Arc::new(Self { task, prev_core }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        let Some(task) = self.task.upgrade() else { return };
        let executor = EXECUTOR.get().expect("Executer not init when wake was called!");
        executor.wake_task(&task, self.prev_core);
    }
}
