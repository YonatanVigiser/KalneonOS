use super::{FrameSize, MemoryMap, MemoryType};
use crate::common::traits::Indexable;

use bitvec::prelude::{BitVec, Lsb0, bitvec};
use x86_64::{PhysAddr, structures::paging::{
    FrameAllocator as Allocator, FrameDeallocator as Deallocator, PageSize, Size4KiB,
    frame::{PhysFrame, PhysFrameRange},
}};

pub const LOW_MEMORY_LIMIT: PhysFrame = unsafe {
    PhysFrame::from_start_address_unchecked(PhysAddr::new(0x100_000))
};

#[derive(Debug)]
pub struct BitmapAllocator {
    frames_bitmap: BitVec<usize, Lsb0>, // 0 is free, 1 is allocated / reserved
    frame_limit: PhysFrame, // Not included
    next_search_hint: usize,
}

impl BitmapAllocator {
    pub fn from_memory_map(mmap: &MemoryMap) -> Self {
        let total_frames = mmap
            .entires()
            .iter()
            .rfind(|f| matches!(f.typ, MemoryType::Usable))
            .unwrap()
            .range
            .end
            .as_index();
        let mut frames_bitmap = bitvec![1; total_frames];
        let frame_limit = PhysFrame::from_index(total_frames);
        for usable_region in mmap
            .entires()
            .iter()
            .filter(|f| matches!(f.typ, MemoryType::Usable))
        {
            frames_bitmap[usable_region.range.start.as_index()..usable_region.range.end.as_index()]
                .fill(false);
        }
        let mut allocator = Self {
            frames_bitmap,
            frame_limit,
            next_search_hint: 0,
        };
        unsafe {
            allocator.mark_as_allocated(super::kernel_phys_range());
        }
        unsafe {
            allocator.mark_as_allocated(PhysFrame::range(
                PhysFrame::from_index(0),
                PhysFrame::from_index(1),
            ));
        } // Reserve the first physical page for BDA
        allocator
    }

    unsafe fn mark_as_allocated(&mut self, range: PhysFrameRange) {
        assert!(
            self.frames_bitmap.get(range.end.as_index() - 1).is_some(),
            "mark as used was called on an out of bound frame range"
        );
        self.frames_bitmap[range.start.as_index()..range.end.as_index()].fill(true);
    }

    pub fn allocate_frame_in(&mut self, range: PhysFrameRange<FrameSize>) -> Option<PhysFrame<FrameSize>> {
        let range = range.start.as_index()..range.end.as_index();
        let index = range.start + self.frames_bitmap.get(range)?.first_zero()?;
        self.frames_bitmap.set(index, true);
        Some(PhysFrame::from_index(index))
    }

    pub fn allocate_frame_below(&mut self, limit: PhysFrame) -> Option<PhysFrame<FrameSize>> {
        let limit = limit.min(self.frame_limit);
        let range = PhysFrameRange { start: PhysFrame::from_index(0), end: limit };
        self.allocate_frame_in(range)
    }

    pub fn free_frames_count(&self) -> usize {
        self.frames_bitmap.count_zeros()
    }

    pub fn free_memory_bytes_count(&self) -> u64 {
        self.free_frames_count() as u64 * FrameSize::SIZE
    }
}

unsafe impl Allocator<FrameSize> for BitmapAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if !self
            .frames_bitmap
            .get(self.next_search_hint)
            .map(|b| *b)
            .unwrap_or(true)
        {
            self.frames_bitmap.set(self.next_search_hint, true);
            self.next_search_hint += 1;
            return Some(PhysFrame::from_index(self.next_search_hint - 1));
        }
        let lower_range = PhysFrameRange { start: PhysFrame::from_index(0), end: LOW_MEMORY_LIMIT };
        let upper_range = PhysFrameRange { start: LOW_MEMORY_LIMIT, end: self.frame_limit };
        let frame = self.allocate_frame_in(upper_range)
            .or_else(|| self.allocate_frame_in(lower_range))?;
        self.next_search_hint = frame.as_index();
        Some(frame)
    }
}

impl Deallocator<FrameSize> for BitmapAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        assert!(
            self.frames_bitmap.get(frame.as_index()).is_some(),
            "dealloc was called on an out of bound frame!"
        );
        debug_assert!(
            self.frames_bitmap.get(frame.as_index()).unwrap(),
            "dealloc was called on a frame that was free in frame allocator"
        );
        self.frames_bitmap.set(frame.as_index(), false);
        self.next_search_hint = frame.as_index();
    }
}
