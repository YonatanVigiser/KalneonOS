use super::frame::{MemoryFrame, MemoryType, FRAME_SIZE};
use alloc::vec::Vec;

// This should be provided by the Arch!
pub struct MemoryRegion {
    pub start: usize,
    pub frames_size: usize,
    pub memory_type: MemoryType,
}

impl MemoryRegion {
    pub fn to_frames(&self) -> impl Iterator<Item = MemoryFrame> {
        (0..self.frames_size).map(|frame_index| MemoryFrame::new(self.memory_type, frame_index * FRAME_SIZE + self.start))
    }
}

pub type MemoryMap = Vec<MemoryRegion>;
