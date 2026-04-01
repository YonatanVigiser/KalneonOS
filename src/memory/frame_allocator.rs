use super::{TypedPhysFrameRange, MemoryType};
use crate::traits::Indexable;

use x86_64::structures::paging::{FrameAllocator as Allocator, FrameDeallocator as Deallocator, Size4KiB, frame::{PhysFrame, PhysFrameRange}, PageSize};
use bitvec::prelude::{BitVec, Lsb0, bitvec};

pub type FrameSize = Size4KiB;

#[derive(Debug)]
pub struct FrameAllocator {
    frames: BitVec<usize, Lsb0>, // 0 is free, 1 is allocated / reserved
    next_search_hint: usize,
    allocated_frames_count: usize,
}

impl FrameAllocator {
    pub fn from_memory_map(mmap: &[TypedPhysFrameRange]) -> Self {
        let total_frames = mmap.last().unwrap().range.start.as_index() + 1;
        let mut frames = bitvec![1; total_frames]; 
        for usable_region in mmap.iter().filter(|f| matches!(f.typ, MemoryType::Usable)) {
            frames[usable_region.range.start.as_index()..usable_region.range.end.as_index()].fill(false);
        }
        Self {
            frames,
            next_search_hint: 0,
            allocated_frames_count: 0,
        }
    }

    pub unsafe fn reserve_range(&mut self, range: PhysFrameRange) {
        assert!(self.frames.get(range.end.as_index() - 1).is_none(), "reserve frame range was called on an out of bound frame range");
        self.frames[range.start.as_index()..range.end.as_index()].fill(true);
    }

    pub fn allocated_frames_count(&self) -> usize {
        self.allocated_frames_count
    }

    pub fn free_frames_count(&self) -> usize {
        self.frames.len() - self.allocated_frames_count
    }

    pub fn free_memory_bytes_count(&self) -> u64 {
        self.free_frames_count() as u64 * FrameSize::SIZE
    }
}

unsafe impl Allocator<FrameSize> for FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if self.next_search_hint > self.frames.len() {
            self.next_search_hint = 0;
        }
        if !self.frames.get(self.next_search_hint).map(|b| *b).unwrap_or(true) {
            self.frames.set(self.next_search_hint, true);
            self.next_search_hint += 1;
            self.allocated_frames_count += 1;
            return Some(PhysFrame::from_index(self.next_search_hint - 1));
        }
        let index = self.frames.first_zero()?;
        self.frames.set(index, true);
        self.next_search_hint = index + 1;
        self.allocated_frames_count += 1;
        Some(PhysFrame::from_index(index))
    }
}

impl Deallocator<FrameSize> for FrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        assert!(self.frames.get(frame.as_index()).is_none(), "dealloc was called on an out of bound frame!");
        debug_assert!(!self.frames.get(frame.as_index()).unwrap(), "dealloc was called on a frame that was free in frame allocator");
        self.frames.set(frame.as_index(), false);
        self.allocated_frames_count -= 1;
        self.next_search_hint = frame.as_index();
    }
}
