#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum MemoryFrameType {
    Usable,
    KernelAddressSpace,
    MMIO,
    Reserved,
}

pub const FRAME_SIZE: usize = 4096;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MemoryFrame {
    start: usize,
    memory_type: MemoryFrameType,
    pub(super) deallocated: bool,
}

impl MemoryFrame {
    pub(super) fn new(memory_type: MemoryFrameType, start: usize) -> Self {
        let start = start & !(FRAME_SIZE - 1);
        Self {
            start,
            memory_type,
            deallocated: false,
        }
    }
    
    pub fn index(&self) -> usize {
        self.start / FRAME_SIZE
    }

    pub fn start(&self) -> usize {
        assert!(!self.deallocated, "A deallocated frame was used after freed!");
        self.start
    }

    pub fn end(&self) -> usize {
        assert!(!self.deallocated, "A deallocated frame was used after freed!");
        self.start + FRAME_SIZE
    }

    pub fn memory_type(&self) -> &MemoryFrameType {
        &self.memory_type
    }
}

/* Uncomment when we have backtracing
impl Drop for MemoryFrame {
    fn drop(&mut self) {
        if !self.deallocated {
            panic!("MemoryFrame was dropped without calling dealloc!");
        }
    }
}
*/

