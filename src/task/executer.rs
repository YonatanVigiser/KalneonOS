use core::task::Waker;

use alloc::{collections::btree_map::BTreeMap, sync::{Arc, Weak}, task::Wake, vec::Vec};
use crossbeam_queue::ArrayQueue;
use spin::{Mutex, Once};

use crate::task::{Task, TaskId};

pub static EXECUTER: Once<Executor> = Once::new();

pub struct Executor {
    tasks: Mutex<BTreeMap<TaskId, Arc<Task>>>,
    tasks_queues: Vec<Arc<ArrayQueue<Weak<Task>>>>,
    waker_cache: Mutex<BTreeMap<TaskId, Waker>>,
}

impl Executor {
    pub fn new(threads_count: usize) -> Self {
        let tasks_queues = Vec
        Self {
            tasks: Mutex::new(BTreeMap::new()),
            tasks_queues: Vec::new(),
            waker_cache: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn spawn(&self, task: Task) {
        let task = Arc::new(task);
        if self.tasks.lock().insert(task.id, task.clone()).is_some() {
            panic!("Task with the same ID was already in tasks!");
        }
        let lightest_queue = self.tasks_queues.iter().min_by(|x, y| x.len().cmp(&y.len())).unwrap();
        lightest_queue.push(Arc::downgrade(&task)).expect("All task queues are full");
    }
}

struct TaskWaker {
    task: Weak<Task>,
    task_queue: Arc<ArrayQueue<Weak<Task>>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.task_queue.push(self.task.clone()).expect("Task pushing failed");
    }
}

