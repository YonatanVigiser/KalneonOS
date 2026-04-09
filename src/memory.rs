pub mod frame_allocator;
pub mod vmm;
pub mod paging;
pub mod heap;
pub mod map;

use x86_64::structures::paging::{PageSize, Page, PhysFrame, Size4KiB, frame::PhysFrameRange, page::PageRange, OffsetPageTable, Size1GiB, Mapper, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};
use spin::Mutex;
use map::MemoryMap;

pub type FrameSize = Size4KiB;

pub const HHDM_START: u64 = 0xffff_8000_0000_0000;

unsafe extern "C" {
    static __phys_start: u8;
    static __phys_end: u8;

    static __text_start: u8;
    static __text_end: u8; static __rodata_start: u8;
    static __rodata_end: u8;

    static __data_start: u8;
    static __data_end: u8;
    
    static mut __bss_start: u8;
    static __bss_end: u8;

    static __stack_buttom: u8;
    static __stack_top: u8;

    static __vma_start: u8;
    static __vma_end: u8;
}

pub fn kernel_phys_range() -> PhysFrameRange {
    let start_addr = &raw const __phys_start as u64;
    let end_addr = &raw const __phys_end as u64;
    PhysFrame::range(PhysFrame::containing_address(PhysAddr::new(start_addr)), PhysFrame::containing_address(PhysAddr::new(end_addr)))
}

pub fn kernel_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __vma_start as u64;
    let end_addr = &raw const __vma_end as u64;
    Page::range(Page::from_start_address(VirtAddr::new(start_addr)).unwrap(), Page::containing_address(VirtAddr::new(end_addr - 1)).next())
}

fn kernel_code_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __text_start as u64;
    let end_addr = &raw const __text_end as u64;
    Page::range(Page::from_start_address(VirtAddr::new(start_addr)).unwrap(), Page::containing_address(VirtAddr::new(end_addr - 1)).next())
}

fn kernel_rodata_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __rodata_start as u64;
    let end_addr = &raw const __rodata_end as u64;
    Page::range(Page::from_start_address(VirtAddr::new(start_addr)).unwrap(), Page::containing_address(VirtAddr::new(end_addr - 1)).next())
}

fn kernel_data_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __data_start as u64 ;
    let end_addr = &raw const __data_end as u64 ;
    Page::range(Page::from_start_address(VirtAddr::new(start_addr)).unwrap(), Page::containing_address(VirtAddr::new(end_addr - 1)).next())
}

fn kernel_bss_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __bss_start as u64 ;
    let end_addr = &raw const __bss_end as u64 ;
    Page::range(Page::from_start_address(VirtAddr::new(start_addr)).unwrap(), Page::containing_address(VirtAddr::new(end_addr - 1)).next())
}

fn kernel_stack_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __stack_buttom as u64 ;
    let end_addr = &raw const __stack_top as u64 ;
    Page::range(Page::from_start_address(VirtAddr::new(start_addr)).unwrap(), Page::containing_address(VirtAddr::new(end_addr - 1)).next())
}

fn hhdm_range(mmap: &MemoryMap) -> PageRange {
    let max_phys_frame = mmap.entires().iter().rfind(|f| matches!(f.typ, MemoryType::Usable)).unwrap().range.end.start_address();
    let max_phys_aligned = max_phys_frame.as_u64().next_multiple_of(Size1GiB::SIZE);
    let virt_end_addr = VirtAddr::new(max_phys_aligned + HHDM_START);
    Page::range(Page::from_start_address(VirtAddr::new(HHDM_START)).unwrap(), Page::containing_address(virt_end_addr))
}

pub fn map_phys_range(phys_range: PhysFrameRange, flags: PageTableFlags) -> Option<PageRange> {
    let virt_range = VMM.lock().as_mut().expect("map() should only be called after memory::init()!").allocate_range(phys_range.len() as usize)?;
    let mut mapper_lock = MAPPER.lock();
    let mapper = mapper_lock.as_mut().expect("map() should only be called after memory::init()!");
    let mut frame_allocator_lock = FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator_lock.as_mut().expect("map() should only be called after memory::init()!");
    for (frame, page) in phys_range.zip(virt_range) {
        unsafe { mapper.map_to(page, frame, flags, frame_allocator).ok()?.flush(); }
    }
    Some(virt_range)
}

pub fn map_mmio_range(phys_mmio_range: PhysFrameRange) -> Option<PageRange> {
    let flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE | PageTableFlags::NO_CACHE | PageTableFlags::WRITE_THROUGH;
    map_phys_range(phys_mmio_range, flags)
}

pub static FRAME_ALLOCATOR: Mutex<Option<frame_allocator::BitmapAllocator>> = Mutex::new(None);
pub static VMM: Mutex<Option<vmm::VirtualMemoryManager>> = Mutex::new(None);
pub static MAPPER: Mutex<Option<OffsetPageTable>> = Mutex::new(None);

pub fn init(mmap: &MemoryMap) {
    heap::init();
    log::info!("Heap was initilized");
    let mut allocator = frame_allocator::BitmapAllocator::from_memory_map(mmap);
    log::info!("Frame allocator was initilized");
    let mapper = unsafe { paging::init(&mut allocator, mmap) };
    let mut allocated_virtual_ranges = alloc::collections::VecDeque::new();
    allocated_virtual_ranges.push_back(Page::range(hhdm_range(mmap).end, kernel_range().start));
    allocated_virtual_ranges.push_back(Page::range(kernel_range().start, Page::containing_address(VirtAddr::new_truncate(u64::MAX))));
    let vmm = vmm::VirtualMemoryManager::new(allocated_virtual_ranges);
    *FRAME_ALLOCATOR.lock() = Some(allocator);
    *VMM.lock() = Some(vmm);
    *MAPPER.lock() = Some(mapper);
    crate::drivers::update_mmio_with_paging();
    log::info!("TEST: {:?}", VMM.lock().as_mut().unwrap().allocate_range(8));
    log::info!("Paging was initilized");
}


#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum MemoryType {
    Usable,
    ApciReclaimable,
    MMIO,
    NVM,
    Reserved,
    Defective,
    Other,
}

use crate::traits::Indexable;
impl<S: PageSize> Indexable for PhysFrame<S> {
    fn as_index(&self) -> usize {
        (self.start_address().as_u64() / S::SIZE) as usize
    }

    fn from_index(index: usize) -> Self {
        PhysFrame::from_start_address(PhysAddr::new(index as u64 * S::SIZE)).unwrap()
    }
}

impl<S: PageSize> Indexable for Page<S> {
    fn as_index(&self) -> usize {
        (self.start_address().as_u64() / S::SIZE) as usize
    }

    fn from_index(index: usize) -> Self {
        Page::from_start_address(VirtAddr::new(index as u64 * S::SIZE)).unwrap()
    }
}
