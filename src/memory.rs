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

unsafe extern "C" {
  static __kernel_start: u8;
  static __kernel_end: u8;
}

pub fn kernel_start_addr() -> usize {
  unsafe { &__kernel_start as *const u8 as usize }
}

pub fn kernel_end_addr() -> usize {
  unsafe { &__kernel_end as *const u8 as usize }
}

pub fn kernel_size() -> usize {
  kernel_end_addr() - kernel_start_addr()
}

pub fn kernel_region() -> region::MemoryRegion {
    region::MemoryRegion { start: frame::FrameAlignedAddress::new(kernel_start_addr()), length: kernel_size() / FRAME_SIZE, memory_type: MemoryType::Usable }
}
