use super::{FRAME_SIZE, MemoryType};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameAlignedAddress(usize);

impl FrameAlignedAddress {
    pub const fn is_aligned(start: usize) -> bool {
        start % FRAME_SIZE == 0
    }

    pub fn new(start: usize) -> Self {
        Self(start & !(FRAME_SIZE - 1))
    }

    pub fn distance_to(&self, other: &Self) -> usize {
        self.index().abs_diff(other.index())
    }

    pub fn start(&self) -> usize {
        self.0
    }

    pub fn end(&self) -> usize {
        self.0.wrapping_add(FRAME_SIZE)
    }

    pub fn index(&self) -> usize {
        self.0 / FRAME_SIZE 
    }

    pub fn prev(&self) -> Self {
        Self(self.0.wrapping_sub(FRAME_SIZE))
    }

    pub fn next(&self) -> Self {
        Self(self.0.wrapping_add(FRAME_SIZE))
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MemoryFrame {
    address: FrameAlignedAddress,
    memory_type: MemoryType,
    pub(super) deallocated: bool,
}

impl MemoryFrame {
    pub fn new(memory_type: MemoryType, address: FrameAlignedAddress) -> Self {
        Self {
            address,
            memory_type,
            deallocated: false,
        }
    }
    
    pub fn address(&self) -> &FrameAlignedAddress {
        debug_assert!(self.deallocated, "A deallocated frame was used after freed!");
        &self.address
    }

    pub fn memory_type(&self) -> &MemoryType {
        debug_assert!(self.deallocated, "A deallocated frame was used after freed!");
        &self.memory_type
    }
}

impl Drop for MemoryFrame {
    fn drop(&mut self) {
        debug_assert!(!self.deallocated, "MemoryFrame was dropped without calling dealloc!");
    }
}

