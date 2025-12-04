pub mod display;
pub mod memory;
pub mod io;

use core::panic::PanicInfo;
use spin::Mutex;
use crate::arch::Arch;
use display::color::Color;
use io::keyboard_manager::KeyboardManager;
use memory::frame_allocator::FrameAllocator;
use linked_list_allocator::LockedHeap;

pub static FRAME_ALLOCATOR: Mutex<Option<FrameAllocator>> = Mutex::new(None);

pub static KEYBOARD_STATE_MANAGER: Mutex<KeyboardManager> = Mutex::new(KeyboardManager::init());

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

use crate::TargetArch;

pub fn kmain() -> ! {
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator::from_mmap(TargetArch::get_memory_map()));
    TargetArch::with_video(|video| {
        writeln!(video.clear(), "Kernel start...");
    });
    loop {
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    TargetArch::panic(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
