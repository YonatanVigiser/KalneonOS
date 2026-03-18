use multiboot2::{BootInformation, BootInformationHeader, MemoryArea, MemoryAreaTypeId, MemoryAreaType};
use crate::memory::frame::FrameAlignedAddress;
use crate::memory::{MemoryType, FRAME_SIZE};
use crate::memory::region::MemoryRegion;
use alloc::vec::Vec;

pub struct BootInfo {
    pub mmap: Vec<MemoryRegion>,
}

impl From<MemoryAreaTypeId> for MemoryType {
    fn from(memory_type: MemoryAreaTypeId) -> Self {
        match memory_type.into() {
            MemoryAreaType::Available | MemoryAreaType::AcpiAvailable => MemoryType::Usable,
            MemoryAreaType::Custom(_) => MemoryType::Other,
            _ => MemoryType::Reserved,
        }
    }
}

pub fn load(boot_magic: u32, boot_info_ptr: u32) -> BootInfo {
    if boot_magic == multiboot2::MAGIC {
        let boot_info = unsafe { BootInformation::load(boot_info_ptr as *const BootInformationHeader).unwrap() };
        if let Some(tag) = boot_info.boot_loader_name_tag() && let Ok(name) = tag.name() {
            log::info!("Kernel booted from {} bootloader!", name);
        }
        let mmap = frames_from_mmap(boot_info.memory_map_tag().expect("No memory map was provided by BIOS!").memory_areas());
        log::info!("MEMORY MAP:\n{:?}", mmap);
        return BootInfo { mmap };
    }
    panic!("No multiboot magic found!");
}


fn frames_from_mmap(memory_areas: &[MemoryArea]) -> Vec<MemoryRegion> {
    let mut frames: Vec<MemoryRegion> = Vec::new();
    for memory_area in memory_areas {
        let memory_type: MemoryType = memory_area.typ().into();
        let mut start = FrameAlignedAddress::new(memory_area.start_address() as usize);
        let mut length = memory_area.size() as usize / FRAME_SIZE;
        if memory_area.size() as usize % FRAME_SIZE != 0 {
            length += 1;
        }
        if let Some(ref last_region) = frames.last() {
            if start > last_region.end() {
                // Hole
                if memory_type == MemoryType::Reserved {
                    if last_region.memory_type == MemoryType::Reserved {
                        frames.last_mut().unwrap().length += last_region.end().distance_to(&start) + length;
                    } else {
                        frames.push(MemoryRegion { start: last_region.end(), length: last_region.end().distance_to(&start) + length, memory_type: MemoryType::Reserved  });
                    }
                    continue;
                } else {
                    frames.push(MemoryRegion { start: last_region.end(), length: last_region.end().distance_to(&start), memory_type: MemoryType::Reserved });
                }
            } else if start < last_region.end() {
                // Overlapping - check priority. On lower, move self. On higher, move last. On
                // equal, merge both
                if last_region.memory_type > memory_type {
                    start = last_region.end();
                } else if last_region.memory_type < memory_type {
                    frames.last_mut().unwrap().length = last_region.start.distance_to(&start);
                } else {
                    frames.last_mut().unwrap().length += length;
                    continue;
                }
            } else if memory_type == last_region.memory_type {
                frames.last_mut().unwrap().length += length;
                continue;
            }
        } else {
            let zero = FrameAlignedAddress::new(0);
            if start != zero {
                frames.push(MemoryRegion { start: zero, length: zero.distance_to(&start), memory_type: MemoryType::Reserved });
            }
        }
        frames.push(MemoryRegion { start, length, memory_type });
    }
    frames
}
