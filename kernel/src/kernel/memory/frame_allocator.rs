use alloc::vec::Vec;
use super::frame::{MemoryFrame, MemoryType, FRAME_SIZE};
use super::map::MemoryMap;

#[derive(Debug)]
pub struct FrameAllocator {
    frames: Vec<usize>, // Bitmap. 0 is free, 1 is allocated / reserved
    next_search_hint: usize,
    allocated_frames_count: usize,
}

impl FrameAllocator {
    pub fn from_memory_map(mmap: &MemoryMap) -> Self {
        let mut frames = Vec::new();
        for (count, frame) in mmap.iter().enumerate() {
            if count % usize::BITS as usize == 0 {
                frames.push(0);
            }
            let is_free = matches!(frame.memory_type, MemoryType::Usable);
            let index = frames.len() - 1;
            frames[index] |= (!is_free as usize) << (count % usize::BITS as usize);
        }
        Self {
            frames,
            next_search_hint: 0,
            allocated_frames_count: 0,
        }
    }

    fn change_nth_frame(&mut self, nth: usize, value: bool) {
        let index = nth / usize::BITS as usize;
        let bit = nth % usize::BITS as usize;
        if value {
            self.frames[index] |= 1 << bit;
        } else {
            self.frames[index] &= !(1 << bit);
        }
    }

    fn get_nth_frame(&self, nth: usize) -> bool {
        self.frames.get(nth / usize::BITS as usize).map(|v| v & (1 << (nth % usize::BITS as usize)) != 0).unwrap_or(true)
    }

    pub fn reserve_frames(&mut self, frames: &[usize]) {
        for frame in frames {
            self.change_nth_frame(frame / FRAME_SIZE, true);
        }
    }

    pub fn alloc(&mut self) -> Option<MemoryFrame> {
        // First, try using the last free frame - as cache:
        if !self.get_nth_frame(self.next_search_hint) {
            self.change_nth_frame(self.next_search_hint, true);
            self.next_search_hint += 1;
            self.allocated_frames_count += 1;
            return Some(MemoryFrame::new(MemoryType::Usable, (self.next_search_hint - 1) * FRAME_SIZE));
        }
        // Only if next not free, try searching all of the bitmap. This runs at O(n)
        for (index, &part) in self.frames.iter().enumerate() {
            if part != usize::MAX {
                for bit in 0..usize::BITS as usize {
                    if ((part >> bit) & 1) == 0 {
                        let frame_index = index * usize::BITS as usize + bit;
                        self.change_nth_frame(frame_index, true);
                        self.next_search_hint = frame_index + 1;
                        self.allocated_frames_count += 1;
                        return Some(MemoryFrame::new(MemoryType::Usable, frame_index * FRAME_SIZE));
                    }
                }
            }
        }
        None
    }

    pub fn alloc_range(&mut self, frames_count: usize) -> Option<Vec<MemoryFrame>> {
        // First, try using the last free frame - as cache - runs at O(1):
        let mut free = true;
        for i in 0..frames_count {
            free &= !self.get_nth_frame(self.next_search_hint + i);
        }
        if free {
            for i in 0..frames_count {
                self.change_nth_frame(self.next_search_hint + i, true);
            }
            self.next_search_hint += frames_count;
            self.allocated_frames_count += frames_count;
            return Some(Vec::from_iter((0..frames_count).map(|i| MemoryFrame::new(MemoryType::Usable, (self.next_search_hint - frames_count + i) * FRAME_SIZE))));
        }
        // Only if next not free, try searching all of the bitmap. This runs at O(n*frames_count)
        let mut free_count = 0;
        let mut count = 0;
        'outer: for part in &self.frames {
            for bit in 0..usize::BITS as usize {
                if ((part >> bit) & 1) == 0 {
                    free_count += 1;
                } else {
                    free_count = 0;
                }
                count += 1;
                if free_count == frames_count {
                    break 'outer;
                }
            }
        }
        if free_count == frames_count {
            for i in 0..frames_count {
                self.change_nth_frame(count - frames_count + i, true);
            }
            self.next_search_hint = count;
            self.allocated_frames_count += frames_count;
            return Some(Vec::from_iter((0..frames_count).map(|i| MemoryFrame::new(MemoryType::Usable, (count - frames_count + i) * FRAME_SIZE))));
        }
        None
    }

    pub fn dealloc(&mut self, frame: &mut MemoryFrame) {
        if frame.deallocated {
            panic!("dealloc was called on a frame that previously dealloacted!");
        }
        if !self.get_nth_frame(frame.index()) {
            panic!("dealloc was called on a frame that was free in frame allocator");
        }
        self.change_nth_frame(frame.index(), false);
        self.allocated_frames_count -= 1;
        self.next_search_hint = frame.index().min(self.next_search_hint);
        frame.deallocated = true;
    }

    pub fn free_frames_count(&self) -> usize {
        self.frames.len() * usize::BITS as usize - self.allocated_frames_count
    }

    pub fn allocated_frames_count(&self) -> usize {
        self.allocated_frames_count
    }
}

