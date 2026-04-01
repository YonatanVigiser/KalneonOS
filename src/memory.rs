pub mod frame_allocator;

use x86_64::structures::paging::frame::{PhysFrame, PhysFrameRange};
use x86_64::structures::paging::PageSize;
use x86_64::PhysAddr;

unsafe extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
}

pub fn kernel_start_addr() -> PhysAddr {
    let addr = unsafe { &__kernel_start as *const u8 as u64 };
    PhysAddr::new(addr)
}

pub fn kernel_end_addr() -> PhysAddr {
    let addr = unsafe { &__kernel_end as *const u8 as u64 };
    PhysAddr::new(addr)
}

pub fn kernel_range() -> PhysFrameRange {
    PhysFrame::range(PhysFrame::containing_address(kernel_start_addr()), PhysFrame::containing_address(kernel_end_addr()))
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum MemoryType {
    Usable,
    MMIO,
    NVM,
    Reserved,
    Defective,
    Other,
}

use crate::traits::Indexable;
impl<S: PageSize> Indexable for PhysFrame<S> {
    fn as_index(&self) -> usize {
        (self.start_address().as_u64() / S::SIZE) as usize
    }

    fn from_index(index: usize) -> Self {
        PhysFrame::from_start_address(PhysAddr::new(index as u64 * S::SIZE)).unwrap()
    }
}

pub struct TypedPhysFrameRange {
    pub range: PhysFrameRange,
    pub typ: MemoryType,
}

use alloc::fmt::Debug;
use core::fmt::{Formatter, Result};
impl Debug for TypedPhysFrameRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "0x{:x} - 0x{:x}: {:?}", self.range.start.start_address(), self.range.end.start_address(), self.typ)
    }
}
