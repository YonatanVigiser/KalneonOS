use super::{MemoryType, FRAME_SIZE};
use super::frame::FrameAlignedAddress;

#[derive(Debug)]
pub struct MemoryRegion {
    pub start: FrameAlignedAddress,
    pub length: usize, // In frames
    pub memory_type: MemoryType,
}

impl MemoryRegion {
    pub fn end(&self) -> FrameAlignedAddress {
        FrameAlignedAddress::new(self.start.start() + self.length * FRAME_SIZE)
    }
}
