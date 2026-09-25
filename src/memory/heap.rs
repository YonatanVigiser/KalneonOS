use spin::mutex::SpinMutex;
use talc::{OomHandler, Span, Talc, Talck};
use x86_64::structures::paging::{PageSize, PageTableFlags};

use crate::common::mutex::debug_mutex::RawDebugMutex;
use crate::common::mutex::interrupt_safe_mutex::RawInterruptSafeMutex;

use super::{FrameSize, allocate};

const BOOT_HEAP_SIZE: usize = 1024 * 1024;
static mut BOOT_HEAP: [u8; BOOT_HEAP_SIZE] = [0; BOOT_HEAP_SIZE];

struct KernelOomHandler;

const HEAP_MEMORY_FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::GLOBAL).union(PageTableFlags::WRITABLE).union(PageTableFlags::NO_EXECUTE);
const HEAP_GROW_MIN_PAGES: usize = 64;

impl OomHandler for KernelOomHandler {
    fn handle_oom(talc: &mut Talc<Self>, layout: core::alloc::Layout) -> Result<(), ()> {
        let needed_bytes = layout.size() + layout.align();
        let pages_needed = needed_bytes.div_ceil(FrameSize::SIZE as usize).max(HEAP_GROW_MIN_PAGES);
        let range = allocate(pages_needed, HEAP_MEMORY_FLAGS).ok_or(())?;
        let base = range.start.start_address().as_mut_ptr::<u8>();
        let size = range.len() as usize * FrameSize::SIZE as usize;
        unsafe { talc.claim(Span::from_base_size(base, size))?; }
        Ok(())
    }
}

#[global_allocator]
static ALLOCATOR: Talck<RawInterruptSafeMutex<RawDebugMutex<SpinMutex<()>>>, KernelOomHandler> = Talc::new(KernelOomHandler).lock();

pub fn init() {
    unsafe {
        ALLOCATOR
            .lock()
            .claim(Span::from_array(&raw mut BOOT_HEAP))
            .expect("Heap allocator init failed!");
    }
}
