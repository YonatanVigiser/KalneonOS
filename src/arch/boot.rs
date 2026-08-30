use crate::{drivers::display::{DisplayInfo, framebuffer::FramebufferInfo}, memory::{
    MemoryType,
    map::{MemoryMap, TypedPhysFrameRange},
}};
use crate::common::traits::Indexable;
use alloc::boxed::Box;
use multiboot2::{
    BootInformation, BootInformationHeader, FramebufferType, MemoryArea, MemoryAreaType, MemoryAreaTypeId
};
use x86_64::PhysAddr;
use x86_64::structures::paging::frame::PhysFrame;

pub struct BootInfo {
    pub mmap: MemoryMap,
    pub framebuffer: Option<DisplayInfo>,
    pub rsdt_addr: PhysAddr,
    pub rsdt_revision: u8,
}

impl From<MemoryAreaTypeId> for MemoryType {
    fn from(value: MemoryAreaTypeId) -> Self {
        match value.into() {
            MemoryAreaType::Available => Self::Usable,
            MemoryAreaType::AcpiAvailable => Self::ApciReclaimable,
            MemoryAreaType::Reserved => Self::Reserved,
            MemoryAreaType::ReservedHibernate => Self::NVM,
            MemoryAreaType::Defective => Self::Defective,
            MemoryAreaType::Custom(_) => Self::Other,
        }
    }
}

pub fn load_boot_info(boot_magic: u32, boot_info_ptr: u32) -> BootInfo {
    if boot_magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            BootInformation::load(boot_info_ptr as *const BootInformationHeader).unwrap()
        };
        if let Some(tag) = boot_info.boot_loader_name_tag()
            && let Ok(name) = tag.name()
        {
            log::info!("Kernel booted from {} bootloader!", name);
        }
        let mmap = frames_from_mmap(
            boot_info
                .memory_map_tag()
                .expect("No memory map was provided by BIOS!")
                .memory_areas(),
        );
        log::info!("MEMORY MAP:\n{:?}", mmap);
        let (rsdt_addr, rsdt_revision) = boot_info
            .rsdp_v2_tag()
            .map(|v2_tag| {
                (
                    PhysAddr::new(v2_tag.xsdt_address() as u64),
                    v2_tag.revision(),
                )
            })
            .or(boot_info.rsdp_v1_tag().map(|v1_tag| {
                (
                    PhysAddr::new(v1_tag.rsdt_address() as u64),
                    v1_tag.revision(),
                )
            }))
            .expect("No RSDP was passeed from bootloader!");
        let framebuffer = boot_info.framebuffer_tag().and_then(|res| res.ok()).map(|tag| { DisplayInfo::from(tag) });
        return BootInfo {
            mmap,
            framebuffer,
            rsdt_addr,
            rsdt_revision,
        };
    }
    panic!("No multiboot magic found!");
}

fn frames_from_mmap(memory_areas: &[MemoryArea]) -> MemoryMap {
    let mut frames = MemoryMap::empty();
    for memory_area in memory_areas {
        let typ: MemoryType = memory_area.typ().into();
        let mut start = PhysFrame::containing_address(PhysAddr::new(memory_area.start_address()));
        let end = PhysFrame::containing_address(PhysAddr::new(memory_area.end_address()));
        if let Some(last) = frames.last() {
            if start > last.range.end {
                if typ == MemoryType::Reserved {
                    if last.typ == MemoryType::Reserved {
                        frames.last_mut().unwrap().range.end = end;
                    } else {
                        frames.append(TypedPhysFrameRange {
                            typ: MemoryType::Reserved,
                            range: PhysFrame::range(last.range.end, end),
                        });
                    }
                    continue;
                } else if last.typ == MemoryType::Reserved {
                    frames.last_mut().unwrap().range.end = start;
                } else {
                    frames.append(TypedPhysFrameRange {
                        typ: MemoryType::Reserved,
                        range: PhysFrame::range(last.range.end, start),
                    });
                }
            } else if typ == last.typ {
                frames.last_mut().unwrap().range.end = end;
                continue;
            } else if start < last.range.end {
                if last.typ > typ {
                    start = last.range.end;
                } else if last.typ < typ {
                    frames.last_mut().unwrap().range.end = start;
                }
            }
        } else {
            let zero = PhysFrame::from_index(0);
            if start != zero {
                frames.append(TypedPhysFrameRange {
                    typ: MemoryType::Reserved,
                    range: PhysFrame::range(zero, start),
                });
            }
        }
        frames.append(TypedPhysFrameRange {
            typ,
            range: PhysFrame::range(start, end),
        });
    }
    frames
}
