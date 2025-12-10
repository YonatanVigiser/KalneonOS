use super::FRAME_ALLOCATOR;
use crate::arch::Arch;
use crate::TargetArch;
use super::memory::frame::{MemoryFrame, FRAME_SIZE};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

pub type ThreadId = usize;

#[derive(Debug)]
pub enum BlockingEvent {
    Keyboard,
}

#[derive(Debug)]
pub enum ThreadState {
    Running(u64),  // Remaning Duration
    Ready,
    Blocked(BlockingEvent),
    Sleeping(u64), // Remaning Duration
    Terminated,
}

pub static SPAWNED_THREADS_COUNT: AtomicUsize = AtomicUsize::new(0);

pub const THREAD_STACK_SIZE: usize = 1024 * 1024; // 1 MB stack

#[derive(Debug)]
pub struct Thread {
    id: ThreadId,
    stack: Vec<MemoryFrame>,
    stack_ptr: usize,
    state: ThreadState,
    priority: u64,
}

impl Thread {
    pub fn new(entry: fn()) -> Self {
        let id = SPAWNED_THREADS_COUNT.fetch_add(1, Ordering::SeqCst); // The first one gets 0!
        let stack = FRAME_ALLOCATOR.lock().as_mut().expect("Frame allocator isn't initialized!").alloc_range(THREAD_STACK_SIZE / FRAME_SIZE).expect("Allocation failed!");
        let mut stack_ptr = stack.last().expect("Shouldn't panic!").end();
        TargetArch::fake_thread_entry_stack(&mut stack_ptr, entry);
        Self {
            id,
            stack,
            stack_ptr,
            state: ThreadState::Ready,
            priority: 0,
        }
    }

    pub unsafe fn context_switch(&mut self, other: &mut Self, new_thread_state: ThreadState) {
        assert!(matches!(self.state, ThreadState::Running), "Attempted to switch context from a non-runnning thread!");
        assert!(matches!(new_thread_state, ThreadState::Running), "Attempted to switch context from a non-runnning thread!");
        self.state = new_thread_state;
        other.state = ThreadState::Running;
        unsafe { crate::TargetArch::context_switch(&mut self.stack_ptr, other.stack_ptr); }
    }

    pub fn id(&self) -> ThreadId {
        self.id
    }

    pub fn stack_ptr(&self) -> usize {
        self.stack_ptr
    }

    pub fn state(&self) -> &ThreadState {
        &self.state
    }

    pub fn priority(&self) -> u64 {
        self.priority
    }

    pub fn increase_priority(&mut self) {
        self.priority += 1;
    }

    pub fn set_priority(&mut self, priority: u64) {
        self.priority = priority;
    }

    pub fn set_state(&mut self, new_state: ThreadState) {
        self.state = new_state;
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        for frame in self.stack.as_mut_slice() {
            FRAME_ALLOCATOR.lock().as_mut().expect("Frame allocator isn't initilized!").dealloc(frame);
        }
    }
}
