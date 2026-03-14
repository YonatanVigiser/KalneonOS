use bitvec::prelude::{BitVec, Lsb0, bitvec};
use super::frame::{MemoryFrame, MemoryType, FRAME_SIZE};

#[derive(Debug)]
pub struct FrameAllocator {
    frames: BitVec<usize, Lsb0>, // 0 is free, 1 is allocated / reserved
    next_search_hint: usize,
    allocated_frames_count: usize,
}

impl FrameAllocator {
    pub fn from_memory_map(mmap: &[MemoryFrame]) -> Self {
        let total_frames = mmap[mmap.len() - 1].index() + 1;
        let mut frames = bitvec![1; total_frames]; 
        for usable_frame in mmap.iter().filter(|f| matches!(f.memory_type(), MemoryType::Usable)) {
            frames.set(usable_frame.index(), false);
        }
        Self {
            frames,
            next_search_hint: 0,
            allocated_frames_count: 0,
        }
    }

    pub fn alloc(&mut self) -> Option<MemoryFrame> {
        if !self.frames.get(self.next_search_hint).expect("Out of bounds bitmap operation!") {
            self.frames.set(self.next_search_hint, true);
            self.next_search_hint += 1;
            self.allocated_frames_count += 1;
            return Some(MemoryFrame::new(MemoryType::Usable, (self.next_search_hint - 1) * FRAME_SIZE));
        }
        let index = self.frames.first_zero()?;
        self.frames.set(index, true);
        self.next_search_hint = index + 1;
        self.allocated_frames_count += 1;
        Some(MemoryFrame::new(MemoryType::Usable, index * FRAME_SIZE))
    }

    pub fn dealloc(&mut self, frame: &mut MemoryFrame) {
        debug_assert!(frame.deallocated, "dealloc was called on a frame that previously dealloacted!");
        assert!(self.frames.get(frame.index()).is_none(), "dealloc was called on an out of bound frame!");
        debug_assert!(!self.frames.get(frame.index()).unwrap(), "dealloc was called on a frame that was free in frame allocator");
        self.frames.set(frame.index(), false);
        self.allocated_frames_count -= 1;
        self.next_search_hint = frame.index().min(self.next_search_hint);
        frame.deallocated = true;
    }

    pub fn free_frames_count(&self) -> usize {
        self.frames.len() - self.allocated_frames_count
    }

    pub fn allocated_frames_count(&self) -> usize {
        self.allocated_frames_count
    }
}

