use x86_64::registers::{
    control::{Cr3, Cr3Flags},
    model_specific::{Efer, EferFlags, Msr},
};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageSize, PageTable, PageTableFlags as Flags, PhysFrame,
    Size1GiB,
    mapper::{MappedPageTable, OffsetPageTable, PageTableFrameMapping},
    page::PageRange,
};
use x86_64::{PhysAddr, VirtAddr};

use super::{FrameSize, HHDM_START, map::MemoryMap};

struct IdentityMapper;

unsafe impl PageTableFrameMapping for IdentityMapper {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        frame.start_address().as_u64() as *mut PageTable
    }
}

pub unsafe fn init(
    allocator: &mut dyn FrameAllocator<FrameSize>,
    mmap: &MemoryMap,
) -> (OffsetPageTable<'static>, PhysFrame) {
    let l4_table_frame = allocator.allocate_frame().unwrap();
    let l4_ptr = l4_table_frame.start_address().as_u64() as *mut PageTable;
    unsafe {
        l4_ptr.write(PageTable::new());
    }
    let mut mapper = unsafe { MappedPageTable::new(&mut *l4_ptr, IdentityMapper) };
    // Map the hhdm
    let max_phys_frame = mmap.entires().last().unwrap().range.end.start_address();
    let max_phys_aligned = max_phys_frame.as_u64().next_multiple_of(Size1GiB::SIZE);
    let hhdm_flags =
        Flags::PRESENT | Flags::GLOBAL | Flags::HUGE_PAGE | Flags::NO_EXECUTE | Flags::WRITABLE;
    for addr in (0..max_phys_aligned).step_by(Size1GiB::SIZE as usize) {
        let frame = PhysFrame::<Size1GiB>::from_start_address(PhysAddr::new(addr)).unwrap();
        let page = Page::<Size1GiB>::from_start_address(VirtAddr::new(addr + HHDM_START)).unwrap();
        unsafe {
            mapper
                .map_to(page, frame, hhdm_flags, allocator)
                .unwrap()
                .ignore();
        }
    }
    // Map the kernel sections
    let vma_to_phys =
        |vma: VirtAddr| PhysAddr::new(vma.as_u64() - &raw const super::__vma_start as u64);
    unsafe {
        map_section(
            &mut mapper,
            allocator,
            vma_to_phys,
            super::kernel_code_range(),
            Flags::PRESENT | Flags::GLOBAL,
        );
        map_section(
            &mut mapper,
            allocator,
            vma_to_phys,
            super::kernel_rodata_range(),
            Flags::PRESENT | Flags::GLOBAL | Flags::NO_EXECUTE,
        );
        map_section(
            &mut mapper,
            allocator,
            vma_to_phys,
            super::kernel_data_range(),
            Flags::PRESENT | Flags::GLOBAL | Flags::NO_EXECUTE | Flags::WRITABLE,
        );
        map_section(
            &mut mapper,
            allocator,
            vma_to_phys,
            super::kernel_bss_range(),
            Flags::PRESENT | Flags::GLOBAL | Flags::NO_EXECUTE | Flags::WRITABLE,
        );
        map_section(
            &mut mapper,
            allocator,
            vma_to_phys,
            super::bsp_stack_range(),
            Flags::PRESENT | Flags::GLOBAL | Flags::NO_EXECUTE | Flags::WRITABLE,
        );
    };
    let l4_hhdm_ptr = (l4_table_frame.start_address().as_u64() + HHDM_START) as *mut PageTable;
    (unsafe { OffsetPageTable::new(&mut *l4_hhdm_ptr, VirtAddr::new(HHDM_START)) }, l4_table_frame)
}

const MSR_PAT_ENTRY: u32 = 0x277;
const PAT_VALUE: u64 = 0x0007010600070106;

pub unsafe fn enable(l4_table_frame: PhysFrame) {
    unsafe {
        Efer::update(|flags| {
            *flags |= EferFlags::NO_EXECUTE_ENABLE;
        });
        Msr::new(MSR_PAT_ENTRY).write(PAT_VALUE);
        Cr3::write(l4_table_frame, Cr3Flags::empty());
    }
}

unsafe fn map_section(
    mapper: &mut impl Mapper<FrameSize>,
    allocator: &mut dyn FrameAllocator<FrameSize>,
    vma_to_phys: impl Fn(VirtAddr) -> PhysAddr,
    vma_range: PageRange<FrameSize>,
    flags: Flags,
) {
    for page in vma_range {
        let frame = PhysFrame::from_start_address(vma_to_phys(page.start_address()))
            .expect("map_section() was called with a bad vma_to_phys mapper function");
        unsafe {
            mapper
                .map_to(page, frame, flags, allocator)
                .unwrap()
                .ignore();
        }
    }
}
