use core::{sync::atomic::{AtomicU64, Ordering}, task::{Context, Poll, Waker}};

use alloc::{collections::btree_map::BTreeMap, sync::{Arc, Weak}, vec::Vec};
use crossbeam_queue::ArrayQueue;
use spin::{Mutex, Once};

use crate::{arch::cpu::{CpuId, current_cpu}, task::{Task, TaskId, TaskState::*, waker::TaskWaker}, time::uptime};

const TASKS_QUEUE_SIZE: usize = 100;

pub static EXECUTOR: Once<Executor> = Once::new();

pub static CORE_SWAPS_COUNT: AtomicU64 = AtomicU64::new(0);

pub struct Executor {
    tasks: Mutex<BTreeMap<TaskId, Arc<Task>>>,
    tasks_queues: Vec<Arc<ArrayQueue<Weak<Task>>>>,
    average_runtimes: Vec<AtomicU64>,
}

impl Executor {
    pub fn init(threads_count: usize) {
        let mut tasks_queues = Vec::with_capacity(threads_count);
        let mut average_runtimes = Vec::with_capacity(threads_count);
        for _ in 0..threads_count {
            tasks_queues.push(Arc::new(ArrayQueue::new(TASKS_QUEUE_SIZE)));
            average_runtimes.push(AtomicU64::new(0));
        }
        EXECUTOR.call_once(|| Self {
            tasks: Mutex::new(BTreeMap::new()),
            tasks_queues,
            average_runtimes,
        });
    }

    pub fn spawn(&self, task: Task) {
        let task = Arc::new(task);
        if self.tasks.lock().insert(task.id, task.clone()).is_some() {
            panic!("Task with the same ID was already in tasks!");
        }
        let lightest_core = self
            .tasks_queues
            .iter()
            .enumerate()
            .min_by(|(_, x), (_, y)| x.len().cmp(&y.len()))
            .unwrap()
            .0;
        task.state.store(Scheduled, Ordering::Release);
        self.push(lightest_core, &task);
    }

    pub fn spawn_in(&self, task: Task, core: CpuId) {
        let task = Arc::new(task);
        if self.tasks.lock().insert(task.id, task.clone()).is_some() {
            panic!("Task with the same ID was already in tasks!");
        }
        task.state.store(Scheduled, Ordering::Release);
        self.push(core.0, &task);
    }

    pub fn run(&self) -> ! {
        let core_id = current_cpu().logical_id;
        let task_queue = &self.tasks_queues[core_id.0];
        let avg = &self.average_runtimes[core_id.0];
        loop {
            if let Some(weak_task) = task_queue.pop() && let Some(task) = weak_task.upgrade() {
                avg.fetch_sub(task.average_runtime_nanos(), Ordering::Relaxed);
                let start_time = uptime();
                self.execute_task(&task);
                let end_time = uptime();
                let delta_time = end_time - start_time;
                task.record_runtime(delta_time);
            }
        }
    }

    fn execute_task(&self, task: &Arc<Task>) {
        task.state.store(Running, Ordering::Release);
        let core_id = current_cpu().logical_id;
        current_cpu().current_task_id.set(Some(task.id));
        let waker: Waker = TaskWaker::new_waker(Arc::downgrade(task), core_id.0);
        let mut context = Context::from_waker(&waker);
        loop {
            match task.poll(&mut context) {
                Poll::Ready(()) => { task.state.store(Completed, Ordering::Release); self.tasks.lock().remove(&task.id).expect("Shouldn't panic!"); break; },
                Poll::Pending => {
                    match task.state.compare_exchange(Running, Idle, Ordering::AcqRel, Ordering::Acquire) {
                        Ok(_) => break,
                        Err(Notified) => {
                            task.state.store(Scheduled, Ordering::Release);
                            self.enqueue(task, core_id.0);
                            break;
                        },
                        Err(other) => panic!("unexpected task state: {:?}", other),
                    }
                },
            }
        }
        current_cpu().current_task_id.set(None);
    }

    fn push(&self, core: usize, task: &Arc<Task>) {
        self.average_runtimes[core].fetch_add(task.average_runtime_nanos(), Ordering::Relaxed);
        self.tasks_queues[core].push(Arc::downgrade(task)).expect("Task queue is full");
    }


    fn enqueue(&self, task: &Arc<Task>, prev_core: usize) {
        let queue = if !task.is_pinned() {
            let iter = self.tasks_queues.iter().zip(self.average_runtimes.iter()).map(|(queue, avg)| avg.load(Ordering::Relaxed) * queue.len() as u64).enumerate();
            let (estimate_sum, count) = iter.clone().fold((0u64, 0u64), |(sum, count), x| (sum + x.1, count + 1));
            let avrage_estimate = estimate_sum / count;
            let current_estimate = self.average_runtimes[prev_core].load(Ordering::Relaxed) * self.tasks_queues[prev_core].len() as u64;
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


    pub(super) fn wake_task(&self, task: &Arc<Task>, prev_core: usize) {
        loop {
            match task.state.compare_exchange_weak(Idle, Scheduled, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => break,
                Err(Running) => {
                    if task.state.compare_exchange(Running, Notified, Ordering::AcqRel, Ordering::Acquire).is_ok() { return }
                }
                Err(Scheduled | Notified) => return,
                Err(Completed) => return,
                Err(Idle) => {}
            }
        }
        self.enqueue(task, prev_core);
    }

    pub fn get_task(&self, id: TaskId) -> Option<Arc<Task>> {
        self.tasks.lock().get(&id).map(|t| t.clone())
    }
}
