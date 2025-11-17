use super::BLOCK_SIZE;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq)]
pub struct MemoryRegion {
    start: usize,
    size: usize,
}

impl MemoryRegion {
    pub fn new(start: usize, size: usize) {
        let start = start - (start % BLOCK_SIZE);
        let size = size - (size % BLOCK_SIZE);
        Self {
            start,
            size,
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn end(&self) -> usize {
        self.start + self.size
    }

    pub fn split(&mut self, new_size: usize) -> Option<Self> {
        if self.size <= new_size {
            return None; 
        }
        self.size -= new_size;
        Some(Self {
            start: self.start + new_size,
            size: new_size,
        })
    }

    pub fn continuos_with(&self, other: &Self) -> bool {
        self.end() == other.start() ||
        other.end() == self.start()
    }

    pub fn join(self, other: Self) -> Result<Self, (Self, Self)> {
        if !self.continuos_with(other) {
            return Err((self, other));
        }
        use core::cmp::min;
        Ok(Self {
            start: min(self.start, other.start),
            size: self.size + other.size,
        })
    }
}
