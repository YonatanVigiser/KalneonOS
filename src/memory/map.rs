use x86_64::structures::paging::frame::{PhysFrame, PhysFrameRange};

use super::MemoryType;
use crate::traits::Indexable;

#[derive(Clone, Copy)]
pub struct TypedPhysFrameRange {
    pub range: PhysFrameRange,
    pub typ: MemoryType,
}

use alloc::fmt::Debug;
use core::fmt::{Formatter, Result};
impl Debug for TypedPhysFrameRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "0x{:x} - 0x{:x}: {:?}",
            self.range.start.start_address(),
            self.range.end.start_address(),
            self.typ
        )
    }
}

pub const MEMORY_MAP_ENTRIES: usize = 256;

pub struct MemoryMap {
    entires: [TypedPhysFrameRange; MEMORY_MAP_ENTRIES],
    occupied_count: usize,
}

impl MemoryMap {
    pub fn empty() -> Self {
        Self {
            entires: [TypedPhysFrameRange {
                range: PhysFrame::range(PhysFrame::from_index(0), PhysFrame::from_index(0)),
                typ: MemoryType::Reserved,
            }; MEMORY_MAP_ENTRIES],
            occupied_count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.occupied_count == 0
    }

    pub fn last(&self) -> Option<&TypedPhysFrameRange> {
        if self.occupied_count == 0 {
            return None;
        }
        Some(&self.entires[self.occupied_count - 1])
    }

    pub fn last_mut(&mut self) -> Option<&mut TypedPhysFrameRange> {
        if self.occupied_count == 0 {
            return None;
        }
        Some(&mut self.entires[self.occupied_count - 1])
    }

    pub fn entires(&self) -> &[TypedPhysFrameRange] {
        &self.entires[..self.occupied_count]
    }

    pub fn append(&mut self, range: TypedPhysFrameRange) {
        assert_ne!(
            self.occupied_count, MEMORY_MAP_ENTRIES,
            "Memory map had too much entires!"
        );
        self.entires[self.occupied_count] = range;
        self.occupied_count += 1;
    }
}

impl Debug for MemoryMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "MemoryMap with {} entires.\n{:?}",
            self.occupied_count,
            self.entires()
        )
    }
}
