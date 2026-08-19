use spin::mutex::SpinMutex;
use talc::{OomHandler, Span, Talc, Talck};

use crate::interrupt::mutex::RawInterruptSafeMutex;

const BOOT_HEAP_SIZE: usize = 1024 * 1024;
static mut BOOT_HEAP: [u8; BOOT_HEAP_SIZE] = [0; BOOT_HEAP_SIZE];

struct KernelOomHandler;

impl OomHandler for KernelOomHandler {
    fn handle_oom(talc: &mut Talc<Self>, layout: core::alloc::Layout) -> Result<(), ()> {
        Err(())
    }
}

#[global_allocator]
static ALLOCATOR: Talck<RawInterruptSafeMutex<SpinMutex<()>>, KernelOomHandler> = Talc::new(KernelOomHandler).lock();

pub fn init() {
    unsafe {
        ALLOCATOR
            .lock()
            .claim(Span::from_array(&raw mut BOOT_HEAP))
            .expect("Heap allocator init failed!");
    }
}
