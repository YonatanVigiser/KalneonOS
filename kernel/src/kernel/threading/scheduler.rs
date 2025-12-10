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
    if SWITCH_MISSED.load(Ordering::Acquire) {
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
        self.threads.iter_mut().filter(|thread| matches!(thread.state(), ThreadState::Ready)).for_each(|thread| thread.priority += 1);
        let delta_time = current_time - self.last_wake_time;
        let mut switch = false;
        for thread in self.threads.as_mut_slice() {
            if let ThreadState::Sleeping(remaning_duration) = thread.state() {
                if remaning_duration - delta_time > 0 {
                    thread.set_state(ThreadState::Sleeping(remaning_duration - delta_time));
                } else {
                    thread.set_state(ThreadState::Ready);
                }
            }
            if let ThreadState::Running(remaning_duration) = thread.state() {
                let new_remaning_duration = remaning_duration.saturating_sub(delta_time);
                if new_remaning_duration == 0 {
                    switch = true;
                }
                thread.set_state(ThreadState::Running(new_remaning_duration));
            }
        }
        self.last_wake_time = current_time;
        // Context switch if needed:
        if switch {
            if preemption_enabled() {
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
        let mut iter = self.threads.iter_mut();
        let current_running_thread = iter.find(|thread| matches!(thread.state(), ThreadState::Running(_)));
        let new_thread = iter.filter(|thread| matches!(thread.state(), ThreadState::Ready)).max_by(|t1, t2| t1.priority.cmp(&t2.priority));
        let idle_thread = self.idle_thread.as_mut().expect("No idle thread was configured for the scheduler!");
        if new_thread.is_some() && current_running_thread.is_some() {
            let new_thread = new_thread.unwrap();
            let current_running_thread = current_running_thread.unwrap();
            new_thread.set_state(ThreadState::Running(THREAD_PREEMPTION_TIME_MS));
            current_running_thread.set_state(old_thread_state);
            unsafe { current_running_thread.context_switch(new_thread); }
        } else if new_thread.is_some() && current_running_thread.is_none() {
            let new_thread = new_thread.unwrap();
            new_thread.set_state(ThreadState::Running(THREAD_PREEMPTION_TIME_MS));
            unsafe { idle_thread.context_switch(new_thread); }
        } else if new_thread.is_none() && current_running_thread.is_some() {
            let current_running_thread = current_running_thread.unwrap();
            current_running_thread.set_state(old_thread_state);
            unsafe { current_running_thread.context_switch(idle_thread); }
        }
    }

    fn cleanup_terminated(&mut self) {
        self.threads.retain(|thread| matches!(thread.state(), ThreadState::Terminated));
    }

    pub fn exit_thread(&mut self) -> ! {
        self.switch_running_thread(ThreadState::Terminated);
        panic!("exit_thread() was called from withing an idle thread!")
    }

    pub fn yield_now(&mut self) {
        self.switch_running_thread(ThreadState::Ready);
    }
    
    pub fn start(&mut self) {
        self.switch_running_thread(ThreadState::Terminated);
        panic!("Scheduler start failed, at least one thread should be added except the idle thread!")
    }

    pub fn set_idle_thread(&mut self, idle_thread: Thread) {
        self.idle_thread = Some(idle_thread);
    }
}
