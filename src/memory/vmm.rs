use crate::common::traits::Indexable;
use alloc::collections::VecDeque;
use x86_64::structures::paging::page::{Page, PageRange};

pub struct VirtualMemoryManager {
    free_ranges: VecDeque<PageRange>,
}

impl VirtualMemoryManager {
    pub fn new(free_ranges: VecDeque<PageRange>) -> Self {
        Self { free_ranges }
    }

    pub fn allocate_page(&mut self) -> Option<Page> {
        let range = self.free_ranges.front_mut()?;
        let prev = range.start;
        range.start = prev.next();
        if range.is_empty() {
            self.free_ranges.pop_front();
        }
        Some(prev)
    }

    pub fn allocate_range(&mut self, size: usize) -> Option<PageRange> {
        let index = self
            .free_ranges
            .iter()
            .position(|range| range.len() >= size as u64)?;
        let range = &mut self.free_ranges[index];
        let prev = range.start;
        range.start = prev.next_nth(size);
        if range.is_empty() {
            self.free_ranges.remove(index);
        }
        Some(Page::range(prev, prev.next_nth(size)))
    }

    pub fn free(&mut self, range: PageRange) {
        assert!(!range.is_empty(), "Trying to free empty page range!");
        if let Some(index_prev) = self
            .free_ranges
            .iter()
            .position(|prev_range| range.start >= prev_range.start)
        {
            let prev_range = self.free_ranges.get_mut(index_prev).unwrap();
            assert!(
                range.start >= prev_range.end,
                "Trying to free a page range that was already (partially) free!"
            );
            let index = if range.start == prev_range.end {
                prev_range.end = range.end;
                index_prev
            } else {
                self.free_ranges.insert(index_prev + 1, range);
                index_prev + 1
            };
            if let Some(next_range) = self.free_ranges.get(index + 1)
                && range.end == next_range.start
            {
                self.free_ranges.get_mut(index).unwrap().end = next_range.end;
            }
        } else {
            self.free_ranges.push_front(range);
        }
    }
}
