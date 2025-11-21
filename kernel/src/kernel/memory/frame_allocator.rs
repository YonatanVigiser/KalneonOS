use super::frame::{MemoryFrame, FRAME_SIZE};
use alloc::collections::linked_list::LinkedList;
use alloc::vec::Vec;

pub struct FrameAllocator {
    free_frames: LinkedList<MemoryFrame>,
}

impl FrameAllocator {
    pub fn new(last_addressable_byte: usize) -> Self {
        let mut list = LinkedList::new();
        for start_address in (0..=last_addressable_byte).step_by(FRAME_SIZE) {
            list.push_front(MemoryFrame::new(start_address));
        }
        Self {
            free_frames: list,
        }
    }

    pub fn reserve_frames(&mut self, frames: &[MemoryFrame]) -> Vec<MemoryFrame> {
        self.free_frames.extract_if(|f| {
            for frame in frames {
                if f == frame {
                    return true;
                }
            }
            false
        }).collect()
    }

    pub fn alloc(&mut self) -> Option<MemoryFrame> {
        self.free_frames.pop_front()
    }

    pub fn dealloc(&mut self, frame: MemoryFrame) {
        self.free_frames.push_front(frame);
    }
}
