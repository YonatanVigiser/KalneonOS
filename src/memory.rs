pub mod heap;
pub mod map;
pub mod frame_allocator;
pub mod vmm;
use map::MemoryMap;
use spin::Mutex;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTableFlags, PhysFrame, Size1GiB,
    Size4KiB, frame::PhysFrameRange, page::PageRange,
};
use x86_64::{PhysAddr, VirtAddr};
pub type FrameSize = Size4KiB;

pub const HHDM_START: u64 = 0xffff_8000_0000_0000;

unsafe extern "C" {
    static __phys_start: u8;
    static __phys_end: u8;

    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;

    static __data_start: u8;
    static __data_end: u8;

    static mut __bss_start: u8;
    static __bss_end: u8;

    static __bsp_stack_buttom: u8;
    static __bsp_stack_top: u8;

    static __vma_start: u8;
    static __vma_end: u8;
}

pub fn kernel_phys_range() -> PhysFrameRange {
    let start_addr = &raw const __phys_start as u64;
    let end_addr = &raw const __phys_end as u64;
    PhysFrame::range(
        PhysFrame::containing_address(PhysAddr::new(start_addr)),
        PhysFrame::containing_address(PhysAddr::new(end_addr)),
    )
}

pub fn kernel_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __vma_start as u64;
    let end_addr = &raw const __vma_end as u64;
    Page::range(
        Page::from_start_address(VirtAddr::new(start_addr)).unwrap(),
        Page::containing_address(VirtAddr::new(end_addr - 1)).next(),
    )
}

pub fn kernel_code_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __text_start as u64;
    let end_addr = &raw const __text_end as u64;
    Page::range(
        Page::from_start_address(VirtAddr::new(start_addr)).unwrap(),
        Page::containing_address(VirtAddr::new(end_addr - 1)).next(),
    )
}

pub fn kernel_rodata_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __rodata_start as u64;
    let end_addr = &raw const __rodata_end as u64;
    Page::range(
        Page::from_start_address(VirtAddr::new(start_addr)).unwrap(),
        Page::containing_address(VirtAddr::new(end_addr - 1)).next(),
    )
}

pub fn kernel_data_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __data_start as u64;
    let end_addr = &raw const __data_end as u64;
    Page::range(
        Page::from_start_address(VirtAddr::new(start_addr)).unwrap(),
        Page::containing_address(VirtAddr::new(end_addr - 1)).next(),
    )
}

pub fn kernel_bss_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __bss_start as u64;
    let end_addr = &raw const __bss_end as u64;
    Page::range(
        Page::from_start_address(VirtAddr::new(start_addr)).unwrap(),
        Page::containing_address(VirtAddr::new(end_addr - 1)).next(),
    )
}

/// Returns the byte offset between VMA and frame_allocator addresses for kernel sections.
pub fn vma_phys_offset() -> u64 {
    &raw const __vma_start as u64
}

pub fn bsp_stack_range() -> PageRange<FrameSize> {
    let start_addr = &raw const __bsp_stack_buttom as u64;
    let end_addr = &raw const __bsp_stack_top as u64;
    Page::range(
        Page::from_start_address(VirtAddr::new(start_addr)).unwrap(),
        Page::containing_address(VirtAddr::new(end_addr - 1)).next(),
    )
}

pub fn hhdm_range(mmap: &MemoryMap) -> PageRange {
    let max_phys_frame = mmap.entires().last().unwrap().range.end.start_address();
    let max_phys_aligned = max_phys_frame.as_u64().next_multiple_of(Size1GiB::SIZE);
    let virt_end_addr = VirtAddr::new(max_phys_aligned + HHDM_START);
    Page::range(
        Page::from_start_address(VirtAddr::new(HHDM_START)).unwrap(),
        Page::containing_address(virt_end_addr),
    )
}

pub fn map_frame(frame: PhysFrame, flags: PageTableFlags) -> Option<Page> {
    map_phys_range(PhysFrame::range(frame, frame.next()), flags).map(|p| p.start)
}

pub fn map_phys_range(phys_range: PhysFrameRange, flags: PageTableFlags) -> Option<PageRange> {
    let virt_range = VMM
        .try_lock()?
        .as_mut()
        .expect("map() should only be called after memory::init()!")
        .allocate_range(phys_range.len() as usize)?;
    let mut mapper_lock = MAPPER.try_lock()?;
    let mapper = mapper_lock
        .as_mut()
        .expect("map() should only be called after memory::init()!");
    let mut frame_allocator_lock = FRAME_ALLOCATOR.try_lock()?;
    let frame_allocator = frame_allocator_lock
        .as_mut()
        .expect("map() should only be called after memory::init()!");
    for (frame, page) in phys_range.zip(virt_range) {
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .ok()?
                .flush();
        }
    }
    Some(virt_range)
}

pub fn map_mmio_range(phys_mmio_range: PhysFrameRange) -> Option<PageRange> {
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::GLOBAL
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_EXECUTE
        | PageTableFlags::NO_CACHE
        | PageTableFlags::WRITE_THROUGH;
    map_phys_range(phys_mmio_range, flags)
}

pub fn map_mmio_ptr(ptr: usize, size: usize) -> Option<usize> {
    let ptr = ptr as u64;
    let size = size as u64;
    let offset = ptr % FrameSize::SIZE;
    let phys_start_frame = PhysFrame::containing_address(PhysAddr::new(ptr));
    let phys_end_frame = PhysFrame::containing_address(PhysAddr::new(ptr + size - 1));
    let range = PhysFrame::range(phys_start_frame, phys_end_frame.next());
    map_mmio_range(range)
        .map(|range| range.start.start_address().as_u64() as usize + offset as usize)
}

pub fn allocate(pages_size: usize, flags: PageTableFlags) -> Option<PageRange> {
    let mut vmm_guard = VMM.try_lock()?;
    let vmm = vmm_guard.as_mut().expect("VMM isn't init");
    let pages = vmm.allocate_range(pages_size)?;
    let mut mapper_guard = MAPPER.try_lock()?;
    let mapper = mapper_guard.as_mut().expect("Paging isn't init");
    for page in pages {
        let mut frame_allocator_guard = FRAME_ALLOCATOR.try_lock()?;
        let frame_allocator = frame_allocator_guard
            .as_mut()
            .expect("Frame allocator isn't init");
        let frame = frame_allocator.allocate_frame()?;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .ok()?
                .flush();
        }
    }
    Some(pages)
}

pub static FRAME_ALLOCATOR: Mutex<Option<frame_allocator::BitmapAllocator>> = Mutex::new(None);
pub static VMM: Mutex<Option<vmm::VirtualMemoryManager>> = Mutex::new(None);
pub static MAPPER: Mutex<Option<OffsetPageTable>> = Mutex::new(None);

pub fn allocate_frame() -> Option<PhysFrame<FrameSize>> {
    FRAME_ALLOCATOR.lock().as_mut()?.allocate_frame()
}

pub fn identity_map_frame(frame: PhysFrame<FrameSize>, flags: PageTableFlags) {
    let mut mapper_guard = MAPPER.lock();
    let mapper = mapper_guard.as_mut().expect("Mapper not init");
    let mut fa_guard = FRAME_ALLOCATOR.lock();
    let fa = fa_guard.as_mut().expect("Frame allocator not init");
    unsafe {
        mapper.identity_map(frame, flags, fa).unwrap().flush();
    }
}

pub fn unmap_page(page: Page<FrameSize>) {
    MAPPER
        .lock()
        .as_mut()
        .expect("Mapper not init")
        .unmap(page)
        .expect("Unmap failed")
        .1
        .flush();
}

pub fn init(mmap: &MemoryMap, post_paging: impl FnOnce()) {
    heap::init();
    log::info!("Heap was initilized");
    let mut allocator = frame_allocator::BitmapAllocator::from_memory_map(mmap);
    log::info!("Frame allocator was initilized");
    let (mapper, l4_table_frame) = unsafe { crate::platform::paging::init(&mut allocator, mmap) };
    let mut allocated_virtual_ranges = alloc::collections::VecDeque::new();
    allocated_virtual_ranges.push_back(Page::range(hhdm_range(mmap).end, kernel_range().start));
    allocated_virtual_ranges.push_back(Page::range(
        kernel_range().start,
        Page::containing_address(VirtAddr::new_truncate(u64::MAX)),
    ));
    let vmm = vmm::VirtualMemoryManager::new(allocated_virtual_ranges);
    *FRAME_ALLOCATOR.lock() = Some(allocator);
    *VMM.lock() = Some(vmm);
    *MAPPER.lock() = Some(mapper);
    unsafe { crate::platform::paging::enable(l4_table_frame) };
    post_paging();
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

use crate::utils::traits::Indexable;
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
