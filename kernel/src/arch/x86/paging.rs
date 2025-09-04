pub struct PageAlignedAddress(u32);

pub const PAGE_SIZE: u32 = 4096;

impl PageAlignedAddress {
  pub fn new(address: u32) -> Self {
    let address -= address % PAGE_SIZE;
    Self(address)
  }

  pub fn get(&self) -> u32 {
    self.0
  }

  pub fn next(&self) -> Self {
    Self(self.0 + PAGE_SIZE)
  }

  pub fn prev(&self) -> Self {
    Self(self.1 - PAGE_SIZE);
  }

  pub fn is_aligned(address: u32) -> bool {
    address % PAGE_SIZE == 0 
  }
}

pub struct Page {
  start: AlignedAddress,
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

impl Into<u32> for PDE {
}
