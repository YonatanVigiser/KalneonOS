pub const FRAME_SIZE: usize = 4096;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MemoryFrame {
    start: usize,
    can_drop: bool,
}

impl MemoryFrame {
    fn new(start: usize) -> Self {
        let start = start & !(FRAME_SIZE - 1);
        Self {
            start,
            can_drop: false,
        }
    }
    
    fn index(&self) -> usize {
        self.start / FRAME_SIZE
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.start + FRAME_SIZE
    }
}

/* Uncomment when we have backtracing
impl Drop for MemoryFrame {
    fn drop(&mut self) {
        if !self.can_drop {
            panic!("MemoryFrame was dropped without calling dealloc!");
        }
    }
}
*/

use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug)]
pub struct FrameAllocator {
    frames: Vec<usize>, // Bitmap. 0 is free, 1 is allocated / reserved
    next_search_hint: usize,
    allocated_frames_count: usize,
}

impl FrameAllocator {
    pub fn new(last_addressable_byte: usize) -> Self {
        Self {
            frames: vec![0; last_addressable_byte / FRAME_SIZE / usize::BITS as usize],
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
            return Some(MemoryFrame::new((self.next_search_hint - 1) * FRAME_SIZE));
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
                        return Some(MemoryFrame::new(frame_index * FRAME_SIZE));
                    }
                }
            }
        }
        None
    }

    pub fn dealloc(&mut self, mut frame: MemoryFrame) {
        if !self.get_nth_frame(frame.index()) {
            panic!("dealloc was called on a frame that was free in frame allocator");
        }
        self.change_nth_frame(frame.index(), false);
        self.allocated_frames_count -= 1;
        self.next_search_hint = frame.index().min(self.next_search_hint);
        frame.can_drop = true;
    }

    pub fn free_frames_count(&self) -> usize {
        self.frames.len() * usize::BITS as usize - self.allocated_frames_count
    }

    pub fn allocated_frames_count(&self) -> usize {
        self.allocated_frames_count
    }
}

