use core::sync::atomic::{Ordering, AtomicUsize, AtomicBool};
use super::thread::{Thread, ThreadState, BlockingEvent};
use super::super::sync::Shared;
use alloc::vec::Vec;

static DISABLE_PREEMPTION: AtomicUsize = AtomicUsize::new(1);

static SWITCH_MISSED: AtomicBool = AtomicBool::new(false);

pub static SCHEDULER: Shared<Scheduler> = Shared::new(Scheduler::new()); 

const THREAD_PREEMPTION_TIME_MS: u64 = 10;

pub fn disable_preemption() {
    DISABLE_PREEMPTION.fetch_add(1, Ordering::Release);
}

pub fn enable_preemption() {
    DISABLE_PREEMPTION.fetch_sub(1, Ordering::Release);
    if preemption_enabled() && SWITCH_MISSED.swap(false, Ordering::AcqRel) {
        SCHEDULER.lock().yield_now();
    }
}

pub fn preemption_enabled() -> bool {
    DISABLE_PREEMPTION.load(Ordering::Acquire) == 0
}

pub struct Scheduler {
    idle_thread: Option<Thread>,
    threads: Vec<Thread>,
    last_wake_time: u64,
}

impl Scheduler {
    const fn new() -> Self { 
        Self {
            idle_thread: None,
            threads: Vec::new(),
            last_wake_time: 0,
        }
    }

    pub fn add_thread(&mut self, thread: Thread) {
        self.threads.push(thread);
    }

    pub fn wake_with_time(&mut self, current_time: u64) {
        self.cleanup_terminated();
        self.threads.iter_mut().filter(|thread| matches!(thread.state(), ThreadState::Ready)).for_each(|thread| { thread.priority = thread.priority.saturating_add(1); });
        let delta_time = current_time - self.last_wake_time;
        let mut switch = false;
        let mut found_running = false;
        for thread in self.threads.as_mut_slice() {
            if let ThreadState::Sleeping(remaning_duration) = thread.state() {
                let new_remaning_duration = remaning_duration.saturating_sub(delta_time);
                if new_remaning_duration > 0 {
                    thread.set_state(ThreadState::Sleeping(new_remaning_duration));
                } else {
                    thread.set_state(ThreadState::Ready);
                }
            }
            if let ThreadState::Running(remaning_duration) = thread.state() {
                found_running = true;
                let new_remaning_duration = remaning_duration.saturating_sub(delta_time);
                if new_remaning_duration == 0 {
                    switch = true;
                }
                thread.set_state(ThreadState::Running(new_remaning_duration));
            }
        }
        self.last_wake_time = current_time;
        // Context switch if needed:
        if switch || !found_running {
            if preemption_enabled() {
                panic!("Reached! switch: {}, preemption: {}, delta time: {}, found_running: {}", switch, preemption_enabled(), delta_time, found_running);
                self.switch_running_thread(ThreadState::Ready);
            } else {
                SWITCH_MISSED.store(true, Ordering::Release);
            }
        }
    }

    pub fn wake_with_event(&mut self, event: BlockingEvent) {
        for thread in self.threads.as_mut_slice() {
            if let ThreadState::Blocked(blocking_event) = thread.state() && *blocking_event == event {
                thread.set_state(ThreadState::Ready);
            }
        }
    }

    pub fn switch_running_thread(&mut self, old_thread_state: ThreadState) {
        let current_thread_index = self.threads.iter().position(|thread| matches!(thread.state(), ThreadState::Running(_)));
        let new_thread_index = self.threads.iter().enumerate().filter(|(_, thread)| matches!(thread.state(), ThreadState::Ready)).max_by_key(|(_, thread)| thread.priority).map(|(index, _)| index);
        let mut idle_thread = self.idle_thread.as_mut().expect("No idle thread was configured for the scheduler!");
        match (current_thread_index, new_thread_index) {
            (Some(current_thread_index), Some(new_thread_index)) => {
                let (current_thread, new_thread) = if current_thread_index < new_thread_index {
                    let (left, right) = self.threads.split_at_mut(new_thread_index);
                    (&mut left[current_thread_index], &mut right[0])
                } else {
                    let (left, right) = self.threads.split_at_mut(current_thread_index);
                    (&mut right[0], &mut left[new_thread_index])
                };
                current_thread.context_switch(new_thread, old_thread_state, THREAD_PREEMPTION_TIME_MS);
            },
            (Some(current_thread_index), None) =>
                self.threads[current_thread_index].context_switch(&mut idle_thread, old_thread_state, THREAD_PREEMPTION_TIME_MS),
            (None, Some(new_thread_index)) => {
                idle_thread.set_state(ThreadState::Running(0));
                idle_thread.context_switch(&mut self.threads[new_thread_index], ThreadState::Ready, THREAD_PREEMPTION_TIME_MS)
            },
            (None, None) => {},
        };
    }

    fn cleanup_terminated(&mut self) {
        self.threads.retain(|thread| !matches!(thread.state(), ThreadState::Terminated));
    }

    pub fn exit_thread(&mut self) -> ! {
        self.switch_running_thread(ThreadState::Terminated);
        panic!("exit_thread() was called from withing an idle thread!")
    }

    pub fn yield_now(&mut self) {
        self.switch_running_thread(ThreadState::Ready);
    }
    
    pub fn start(&mut self) -> ! {
        enable_preemption();
        enable_preemption(); // To make up for SCHEDLER lock
        loop {
            core::hint::spin_loop()
        }
    }

    pub fn set_idle_thread(&mut self, idle_thread: Thread) {
        self.idle_thread = Some(idle_thread);
    }
}
