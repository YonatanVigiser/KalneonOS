use core::sync::atomic::{Ordering, AtomicUsize, AtomicBool};
use super::thread::{Thread, ThreadState, BlockingEvent};
use alloc::vec::Vec;

static DISABLE_PREEMPTION: AtomicUsize = AtomicUsize::new(0);

static SWITCH_MISSED: AtomicBool = AtomicBool::new(false);

pub fn default_idle_thread() {
    loop {
        core::hint::spin_loop()
    }
}

pub fn disable_preemption() {
    DISABLE_PREEMPTION.fetch_add(1, Ordering::Release);
}

pub fn enable_preemption() {
    DISABLE_PREEMPTION.fetch_sub(1, Ordering::Release);
    if SWITCH_MISSED.load(Ordering::Acquire) && preemption_enabled() {
        yield_now();
    }
}

pub fn preemption_enabled() -> bool {
    DISABLE_PREEMPTION.load(Ordering::Acquire) == 0
}

pub struct Scheduler {
    idle_thread: Thread,
    threads: Vec<Thread>,
    last_wake_time: u64,
}

impl Scheduler {
    pub fn new(idle_thread: Thread, current_time: u64) -> Self { 
        Self {
            idle_thread,
            threads: Vec::new(),
            last_wake_time: current_time,
        }
    }

    // Should be called each timer intterupt!
    pub fn wake_with_time(&mut self, current_time: u64) {
        // Increament priority for already ready threads:
        self.threads.iter_mut().for_each(|(thread, priority)| if matches!(thread.state(), ThreadState::Ready) { *priority += 1 });
        // Wake threads:
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
                thread.set_state(ThreadState::Running(remaning_duration.saturating_sub(delta_time)));
                if remaning_duration.saturating_sub(delta_time) == 0 {
                    switch = true;
                }
            }
        }
        self.last_wake_time = current_time;
        // Context switch if needed:
        if switch {

        }
        if preemption_enabled() {
            let mut iter = self.threads.iter_mut();
            let current_running_thread = iter.find(|(thread, _)| matches!(thread.state(), ThreadState::Running)).map(|(thread, _)| thread);
            let new_thread = iter.filter(|(thread, _)| matches!(thread.state(), ThreadState::Ready)).max_by(|(_, p1), (_, p2)| p1.cmp(p2)).map(|(thread, _)| thread);
            if new_thread.is_some() && current_running_thread.is_some() {
                unsafe { current_running_thread.unwrap().context_switch(new_thread.unwrap(), ThreadState::Ready); } 
            } else if new_thread.is_some() && current_running_thread.is_none() {
                unsafe { self.idle_thread.context_switch(new_thread.unwrap(), ThreadState::Ready); } 
            } else if new_thread.is_none() && current_running_thread.is_some() {
                unsafe { current_running_thread.unwrap().context_switch(&mut self.idle_thread, ThreadState::Ready); } 
            }
        }
    }
    
    // Should be called in relevant intterupts:
    pub fn wake_with_event(&mut self, Blocki

    pub fn set_idle_thread(&mut self, idle_thread: Thread) {
        self.idle_thread = idle_thread;
    }
}

pub fn yield_now() {
    
}
