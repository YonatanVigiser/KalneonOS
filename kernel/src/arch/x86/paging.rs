#[derive(Clone, Copy)]
pub struct PageAlignedAddress(u32);

pub const PAGE_SIZE: u32 = 4096;

impl PageAlignedAddress {
  pub fn new(mut address: u32) -> Self {
    address -= address % PAGE_SIZE;
    Self(address)
  }

  pub fn get(&self) -> u32 {
    self.0
  }

  pub fn next(&self, n: u32) -> Self {
    Self(self.0 + PAGE_SIZE * n)
  }

  pub fn prev(&self, n: u32) -> Self {
    Self(self.0 - PAGE_SIZE * n)
  }

  pub fn is_aligned(address: u32) -> bool {
    address % PAGE_SIZE == 0 
  }
}

pub struct Page {
  start: PageAlignedAddress,
}

pub struct PDE {
  avaiable: u8,
  global: bool,
  cache_disable: bool,
  write_through: bool,
  user: bool,
  write_enabled: bool,
  present: bool,
}

impl PDE {
  pub fn new() -> Self {
    PDE {
      avaiable: 0,
      global: false,
      cache_disable: false,
      write_through: false,
      user: false,
      write_enabled: false,
      present: false,
    }
  }
}

/*
impl Into<u32> for PDE {
}
*/
