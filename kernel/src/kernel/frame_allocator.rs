use crate::arch::x86::paging::{self, PageAlignedAddress};
use crate::boot::boot_info::MemoryMapEntry;

pub struct Frame {
    start: PageAlignedAddress,
    end: PageAlignedAddress,
    size: u32,
    next_frame: Option<Frame>,
}

impl Iterator for Frame {
   type Item = Frame;
   fn next(&mut self) -> Option<&mut Self::Item> {
       &mut self.next_frame
   }
}

impl Frame {
    pub fn get_start(&self) -> PageAlignedAddress {
        self.start
    }

    pub fn get_end(&self) -> PageAlignedAddress {
        self.end
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}

#[derive(Clone)]
pub struct FrameAllocator(Frame);

impl FrameAllocator {
  pub fn new(mem_map: [MemoryMapEntry; 128]) -> Self {
    let mut count = 0;
    let fisrt = false;
    let first_frame;
    let next_frame;
    for entry in MemoryMapEntry.iter() {
      if entry.mem_type != 0 {
          continue;
      }
      let start = PageAlignedAddress(entry.start);
      let end = PageAlignedAddress(entry.end);
      if !PageAlignedAddress.is_aligned(entry.start) {
        let start = start.next(1);
      }
      if !PageAlignedAddress.is_aligned(entry.end) {
        let end = end.prev(1);
      }

      if first {

          first = false;
      } else {

      }
      let next_frame = Frame {
          start,
          end,
          size: (end.get() - start.get()) / paging::PAGE_SIZE,
          next_frame: None,
      };
    }
    Self(frame)
  }

  pub fn allocate(&mut self, size: u32) -> Option<Frame> {
      if size == 0 {
        return None;
      }
      for frame in self.0 {
          if frame.size >= size {
              if frame.size > size {
                  let split_address = frame.start.next(size);
                  let new_frame = Frame {
                      start: frame.start,
                      end: split_address,
                      size,
                  };
                  frame.start = split_address;
                  frame.size -= size;
                  return Some(new_frame);
              }
              return Some(frame.clone());
          }
      }
      None
  }

  pub fn free(&mut self, frame: Frame) {
      for free_frame in self.0 {
          if free_frame.
      }
  }
}
