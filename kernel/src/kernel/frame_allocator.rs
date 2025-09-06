use crate::arch::x86::paging::{self, PAGE_SIZE, PageAlignedAddress};
use crate::boot::boot_info::MemoryMapEntry;
use crate::utils::heapless::linked_list::LinkedList;

pub struct Frame {
    start: PageAlignedAddress,
    end: PageAlignedAddress,
    size: u32,
}

impl Frame {
    pub fn start(&self) -> PageAlignedAddress {
        self.start
    }

    pub fn end(&self) -> PageAlignedAddress {
        self.end
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

#[derive(Clone)]
pub struct FrameAllocator(LinkedList<Frame>);

impl FrameAllocator {
    pub fn new(mem_map: [MemoryMapEntry; 128]) -> Self {
        let linked_list = LinkedList::new();
        for entry in mem_map.iter() {
            if entry.mem_type != 0 {
                continue;
            }
            let start = PageAlignedAddress::new(entry.base);
            let end = PageAlignedAddress::new(entry.base + entry.length);
            if !PageAlignedAddress.is_aligned(entry.base) {
                let start = start.next(1);
            }
            if !PageAlignedAddress.is_aligned(entry.base + entry.length) {
                let end = end.prev(1);
            }
            linked_list.insert(
                linked_list.size(),
                Frame {
                    start,
                    end,
                    size: (end.get() - start.get()) / PAGE_SIZE,
                },
            );
        }
        Self(linked_list)
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
                let new_frame = frame.clone();
                return Some(new_frame);
            }
        }
        None
    }

    pub fn free(&mut self, frame: Frame) {
        let mut count = 0;
        for free_frame in self.0 {
            if frame.end.get() == free_frame.start.get() {
                free_frame.start = free_frame.prev(frame.size.get());
                free_frame.size += frame.size.get();
                return;
            }
            if frame.end.get() > free_frame.start.get() {
                self.0.insert(count, frame);
            }
            count += 1;
        }
    }
}
