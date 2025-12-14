use crate::kernel::FRAME_ALLOCATOR;
use crate::arch::Arch;
use crate::TargetArch;
use crate::kernel::memory::frame::{MemoryFrame, FRAME_SIZE};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

pub type ThreadId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockingEvent {
    Keyboard,
}

#[derive(Debug)]
pub enum ThreadState {
    Running(u64),  // Remaining Duration
    Ready,
    Blocked(BlockingEvent),
    Sleeping(u64), // Remaining Duration
    Terminated,
}

static SPAWNED_THREADS_COUNT: AtomicUsize = AtomicUsize::new(0);

const THREAD_STACK_SIZE: usize = 1024 * 1024; // 1 MB stack

#[derive(Debug)]
pub struct Thread {
    pub(super) id: ThreadId,
    stack: Vec<MemoryFrame>,
    stack_ptr: usize,
    pub(super) state: ThreadState,
    pub(super) priority: u64,
}

impl Thread {
    pub fn new(entry: fn() -> !) -> Self {
        let id = SPAWNED_THREADS_COUNT.fetch_add(1, Ordering::SeqCst);
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

    pub(super) fn context_switch(&mut self, to: &mut Self, old_thread_state: ThreadState, running_thread_duration: u64) {
        assert!(matches!(self.state, ThreadState::Running(_)), "Attempted to switch context from a non-runnning thread!");
        self.state = old_thread_state;
        to.state = ThreadState::Running(running_thread_duration);
        to.priority = 0;
        unsafe { crate::TargetArch::context_switch(&mut self.stack_ptr, to.stack_ptr); }
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        for frame in self.stack.as_mut_slice() {
            FRAME_ALLOCATOR.lock().as_mut().expect("Frame allocator isn't initilized!").dealloc(frame);
        }
    }
}
