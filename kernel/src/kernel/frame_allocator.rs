use crate::arch::x86::paging::{PAGE_SIZE, PageAlignedAddress};
use crate::boot::boot_info::MemoryMapEntry;
use heapless::sorted_linked_list::{Max, SortedLinkedList};

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
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

const LINKED_LIST_MAX_SIZE: usize = 1024;

pub struct FrameAllocator(SortedLinkedList<Frame, Max, LINKED_LIST_MAX_SIZE, usize>);

impl FrameAllocator {
    pub fn new(mem_map: &[MemoryMapEntry]) -> Self {
        let mut linked_list = SortedLinkedList::new_usize();
        for entry in mem_map.iter() {
            if entry.mem_type != 0 {
                continue;
            }
            let mut start = PageAlignedAddress::new(entry.base as u32);
            let mut end = PageAlignedAddress::new(entry.base as u32 + entry.length as u32);
            if !PageAlignedAddress::is_aligned(entry.base as u32) {
                start = start.next(1);
            }
            if !PageAlignedAddress::is_aligned(entry.base as u32 + entry.length as u32) {
                end = end.prev(1);
            }
            linked_list.push(
                Frame {
                    start,
                    end,
                    size: (end.get() - start.get()) / PAGE_SIZE,
                },
            ).unwrap();
        }
        Self(linked_list)
    }

    pub fn allocate(&mut self, size: u32) -> Option<Frame> {
        if size == 0 {
            return None;
        }
        let frame_index = self.0.iter().position(|frame| { frame.size >= size })?;
        let mut frame = self.linked_list_pop_n(frame_index)?;
        if frame.size != size {
            let mut new_frame = frame.clone();
            new_frame.size -= size;
            new_frame.start = frame.start.next(size);
            frame.size = size;
            frame.end = new_frame.start;
            self.0.push(new_frame).ok();
        }
        Some(frame)
    }

    fn linked_list_pop_n(&mut self, index: usize) -> Option<Frame> {
        let mut list: [Frame; LINKED_LIST_MAX_SIZE - 1] = [Frame {
            start: PageAlignedAddress::new(0),
            end: PageAlignedAddress::new(0),
            size: 0,
        }; LINKED_LIST_MAX_SIZE - 1];
        for i in 0..index {
            list[i] = self.0.pop()?;
        }
        for i in 0..(index - 1) {
            self.0.push(list[i]).ok()?;
        }
        Some(list[index])
    }
    
    pub fn free(&mut self, mut frame: Frame) -> Result<(), Frame> {
        let merge_front = self.0.iter().position(|test_frame| test_frame.start == frame.end);
        let merge_back = self.0.iter().position(|test_frame| test_frame.end == frame.start);
        if merge_front.is_none() && merge_back.is_none() {
            return Err(frame);
        }

        if let Some(frame_index) = merge_back {
            let frame_to_merge = self.linked_list_pop_n(frame_index).unwrap();
            frame.size += frame_to_merge.size;
            frame.start = frame_to_merge.start;
        }

        if let Some(frame_index) = merge_front {
            let frame_to_merge = self.linked_list_pop_n(frame_index).unwrap();
            frame.size += frame_to_merge.size;
            frame.end = frame_to_merge.end;
        }

        self.0.push(frame)?;

        Ok(())
    }
}
