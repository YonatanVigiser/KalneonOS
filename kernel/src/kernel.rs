pub mod display;
pub mod memory;
pub mod io;
pub mod thread;
pub mod scheduler;
pub mod sync;

use core::panic::PanicInfo;
use spin::Mutex;
use crate::arch::Arch;
use io::keyboard_manager::KeyboardManager;
use memory::map::MemoryMap;
use memory::frame_allocator::FrameAllocator;
use linked_list_allocator::LockedHeap;

pub static FRAME_ALLOCATOR: Mutex<Option<FrameAllocator>> = Mutex::new(None);

pub static KEYBOARD_STATE_MANAGER: Mutex<KeyboardManager> = Mutex::new(KeyboardManager::init());

pub static MEMORY_MAP: Mutex<Option<MemoryMap>> = Mutex::new(None);

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static UPTIME_MS: Mutex<u64> = Mutex::new(0);

use crate::TargetArch;

pub fn kmain() -> ! {
    TargetArch::with_video(|video| {
        writeln!(video.clear(), "Kernel start...");
    });

    *MEMORY_MAP.lock() = Some(TargetArch::take_memory_map()).expect("No memory map provided by Arch!");
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator::from_memory_map(MEMORY_MAP.lock().as_mut().unwrap()));
    loop {
    }
}

pub fn idle_thread() {

}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    TargetArch::panic(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
