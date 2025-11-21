pub const FRAME_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct MemoryFrame {
    start: *mut u8,
}

impl MemoryFrame {
    pub fn new(start: usize) -> Self {
        let start = (start - (start % FRAME_SIZE)) as *mut u8;
        Self {
            start,
        }
    }

    pub fn start(&self) -> *mut u8 {
        self.start
    }

    pub fn end(&self) -> *mut u8 {
        (self.start as usize + FRAME_SIZE) as *mut u8
    }
}
