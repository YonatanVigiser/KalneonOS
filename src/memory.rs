pub mod frame;
pub mod region;
pub mod frame_allocator;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum MemoryType {
    Usable,
    MMIO,
    Reserved,
    Defective,
    Other,
}

pub const FRAME_SIZE: usize = 4096;

