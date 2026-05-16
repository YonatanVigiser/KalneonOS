use core::{sync::atomic::{AtomicU64, Ordering}, task::{Context, Poll, Waker}};

use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use crossbeam_queue::ArrayQueue;
use spin::{Mutex, Once};

use crate::{arch::cpu::current_cpu, task::{Task, TaskId, executor, waker::TaskWaker, yield_now}, time::uptime_nano};

const TASKS_QUEUE_SIZE: usize = 100;
const DEFAULT_AVRAGE: u64 = 50_000;
const EWMA_CONSTANT: f64 = 0.05;

pub static EXECUTOR: Once<Executor> = Once::new();

pub static CORE_SWAPS_COUNT: AtomicU64 = AtomicU64::new(0);

pub struct Executor {
    tasks: Mutex<BTreeMap<TaskId, Arc<Task>>>,
    tasks_queues: Vec<Arc<ArrayQueue<Weak<Task>>>>,
    avrage_runtimes: Vec<AtomicU64>,
}

impl Executor {
    pub fn init(threads_count: usize) {
        let mut tasks_queues = Vec::with_capacity(threads_count);
        let mut avrage_runtime = Vec::with_capacity(threads_count);
        for _ in 0..threads_count {
            tasks_queues.push(Arc::new(ArrayQueue::new(TASKS_QUEUE_SIZE)));
            avrage_runtime.push(AtomicU64::new(DEFAULT_AVRAGE));
        }
        EXECUTOR.call_once(|| Self {
            tasks: Mutex::new(BTreeMap::new()),
            tasks_queues,
            avrage_runtimes: avrage_runtime,
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

    pub fn spawn_in(&self, task: Task, core: usize) {
        let queue = self.tasks_queues.get(core).expect("Given core ID is out of bounds!");
        let task = Arc::new(task);
        if self.tasks.lock().insert(task.id, task.clone()).is_some() {
            panic!("Task with the same ID was already in tasks!");
        }
        queue.push(Arc::downgrade(&task)).expect("Queue task is full");
    }

    pub fn run(&self) -> ! {
        let core_id = current_cpu().logical_id;
        let task_queue = &self.tasks_queues[core_id];
        let avg = &self.avrage_runtimes[core_id];
        loop {
            if let Some(weak_task) = task_queue.pop() && let Some(task) = weak_task.upgrade() {
                let start_time = uptime_nano();
                self.execute_task(&task);
                let end_time = uptime_nano();

                let delta_time = (end_time - start_time) as f64;
                let old_time = avg.load(Ordering::Relaxed) as f64;
                let new_avg = old_time * (1.0 - EWMA_CONSTANT) + delta_time * EWMA_CONSTANT;
                avg.store(new_avg as u64, Ordering::Relaxed);
            }
        }
    }

    fn execute_task(&self, task: &Arc<Task>) {
        let core_id = current_cpu().logical_id;
        let waker: Waker = TaskWaker::new_waker(Arc::downgrade(task), core_id);
        let mut context = Context::from_waker(&waker);
        if let Poll::Ready(()) = task.poll(&mut context) {
            self.tasks
                .lock()
                .remove(&task.id)
                .expect("Shouldn't panic!");
        }
    }

    pub(super) fn wake_task(&self, task: &Arc<Task>, prev_core: usize) {
        let queue = if !task.is_pinned() {
            let iter = self.tasks_queues.iter().zip(self.avrage_runtimes.iter()).map(|(queue, avg)| avg.load(Ordering::Relaxed) * queue.len() as u64).enumerate();
            let (estimate_sum, count) = iter.clone().fold((0u64, 0u64), |(sum, count), x| (sum + x.1, count + 1));
            let avrage_estimate = estimate_sum / count;
            let current_estimate = self.avrage_runtimes[prev_core].load(Ordering::Relaxed) * self.tasks_queues[prev_core].len() as u64;
            if current_estimate as f64 / avrage_estimate as f64 > task.affinity_threshold() {
                CORE_SWAPS_COUNT.fetch_add(1, Ordering::Relaxed);
                let min_core = iter.min_by_key(|(_, val)| *val).unwrap().0;
                &self.tasks_queues[min_core]
            } else {
                &self.tasks_queues[prev_core]
            }
        } else {
            &self.tasks_queues[prev_core]
        };
        queue.push(Arc::downgrade(task)).expect("Task queue is full");
    }
}
