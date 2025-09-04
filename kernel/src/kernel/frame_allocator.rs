use core::sync::atomic::AtomicBool;
use crate::arch::x86::paging::PageAlignedAddress;
use crate::boot::boot_info::MemoryMapEntry;

pub struct FrameAllocator {
  used_bitmap: 'static mut [u32],
}

impl FrameAllocator {
  pub fn new(mem_map: MemoryMapEntry[128]) -> Self {
    for entry in MemoryMapEntry.iter() {
      let is_free = entry.mem_type == 0;
      let start = PageAlignedAddress(entry.start);
      let end = PageAlignedAddress(entry.end);
      if is_free && !PageAlignedAddress.is_aligned(entry.start) {
        let start = start.next();
      }
      if !is_free && !PageAlignedAddress.is_aligned(entry.end) {
        let end = end.next();
      }
    }
  }
}
