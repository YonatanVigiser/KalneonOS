use core::fmt;
use crate::debug_hex;

#[repr(C)]
#[derive(Clone)]
pub struct MemoryMapEntry {
  pub base: u64,
  pub length: u64,
  pub mem_type: u32,
  pub extended_attributes: u32,
}

debug_hex!(MemoryMapEntry,
  hex: [base, length, mem_type, extended_attributes],
  normal: []
);

#[repr(C)]
#[derive(Clone)]
pub struct BootInfoBlock {
  pub magic: u32, // Should be "YOVI"
  pub version: u16,
  pub flags: u8,
  pub reserved: [u8; 7], // For future use
  pub boot_disk: u8,
  pub mmap_entry_count: u8,
  pub kernel_base: u64,
  pub kernel_length: u64,
  pub mmap: [MemoryMapEntry; 128],
}

debug_hex!(BootInfoBlock,
  hex: [magic, version, flags, boot_disk, kernel_base, kernel_length],
  normal: [mmap_entry_count, mmap]
);


// Flags:
// 0: Mmap EAB supported
// 1: CPUID supported
// 2-7: reserved

impl BootInfoBlock {
  pub unsafe fn copy_from_ptr(ptr: u32) -> Self {
    let block = unsafe { (*(ptr as *const BootInfoBlock)).clone() };
    assert!(block.magic == 0x594F5649, "Boot Info Block is corrupted!");
    block
  }

  pub fn memory_map(&self) -> &[MemoryMapEntry] {
    &self.mmap[..self.mmap_entry_count as usize]
  }
}
