use core::task::{Context, Poll, Waker};

use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
    task::Wake,
    vec::Vec,
};
use crossbeam_queue::ArrayQueue;
use spin::{Mutex, Once};

use crate::task::{Task, TaskId};

const TASKS_QUEUE_SIZE: usize = 100;

pub static EXECUTER: Once<Executor> = Once::new();

pub struct Executor {
    tasks: Mutex<BTreeMap<TaskId, Arc<Task>>>,
    tasks_queues: Vec<Arc<ArrayQueue<Weak<Task>>>>,
}

impl Executor {
    pub fn init(threads_count: usize) {
        let mut tasks_queues = Vec::with_capacity(threads_count);
        for _ in 0..threads_count {
            tasks_queues.push(Arc::new(ArrayQueue::new(TASKS_QUEUE_SIZE)));
        }
        EXECUTER.call_once(|| Self {
            tasks: Mutex::new(BTreeMap::new()),
            tasks_queues,
        });
    }

    pub fn spawn(&self, task: Task) {
        let task = Arc::new(task);
        if self.tasks.lock().insert(task.id, task.clone()).is_some() {
            panic!("Task with the same ID was already in tasks!");
        }
        let lightest_queue = self
            .tasks_queues
            .iter()
            .min_by(|x, y| x.len().cmp(&y.len()))
            .unwrap();
        lightest_queue
            .push(Arc::downgrade(&task))
            .expect("All task queues are full");
    }

    fn run_ready(&self) {
        let core_id = crate::platform::cpu::current_cpu().logical_id;
        let task_queue = &self.tasks_queues[core_id];
        while let Some(weak_task) = task_queue.pop()
            && let Some(task) = weak_task.upgrade()
        {
            let waker: Waker = TaskWaker::new_waker(Arc::downgrade(&task), task_queue.clone());
            let mut context = Context::from_waker(&waker);
            if let Poll::Ready(()) = task.poll(&mut context) {
                self.tasks
                    .lock()
                    .remove(&task.id)
                    .expect("Shouldn't panic!");
            }
        }
    }

    fn try_steal(&self) -> bool {
        let core_id = crate::platform::cpu::current_cpu().logical_id;
        let task = self
            .tasks_queues
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != core_id)
            .map(|(_, q)| q)
            .find(|q| !q.is_empty())
            .and_then(|q| q.pop());
        if let Some(task) = task {
            self.tasks_queues[core_id]
                .push(task)
                .expect("Current queue is full");
            true
        } else {
            false
        }
    }

    fn sleep(&self) {
        let core_id = crate::platform::cpu::current_cpu().logical_id;
        while self.tasks_queues[core_id].is_empty() {
            core::hint::spin_loop();
        }
    }

    pub fn run(&self) -> ! {
        loop {
            self.run_ready();
            if !self.try_steal() {
                self.sleep();
            }
        }
    }
}

struct TaskWaker {
    task: Weak<Task>,
    task_queue: Arc<ArrayQueue<Weak<Task>>>,
}

impl TaskWaker {
    fn new_waker(task: Weak<Task>, task_queue: Arc<ArrayQueue<Weak<Task>>>) -> Waker {
        Waker::from(Arc::new(Self { task, task_queue }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.task_queue
            .push(self.task.clone())
            .expect("Task pushing failed");
    }
}
