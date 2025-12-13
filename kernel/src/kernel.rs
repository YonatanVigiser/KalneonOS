pub mod display;
pub mod memory;
pub mod io;
pub mod threading;
pub mod sync;

use core::panic::PanicInfo;
use spin::Mutex;
use crate::arch::Arch;
use io::keyboard_manager::KeyboardManager;
use memory::map::MemoryMap;
use memory::frame_allocator::FrameAllocator;
use linked_list_allocator::LockedHeap;
use threading::scheduler::{self, SCHEDULER};
use threading::thread::Thread;
use sync::Shared;

pub static FRAME_ALLOCATOR: Shared<Option<FrameAllocator>> = Shared::new(None);

pub static KEYBOARD_STATE_MANAGER: Shared<KeyboardManager> = Shared::new(KeyboardManager::init());

pub static MEMORY_MAP: Shared<Option<MemoryMap>> = Shared::new(None);

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static UPTIME_MS: Shared<u64> = Shared::new(0);

use crate::TargetArch;

pub fn kmain() -> ! {
    TargetArch::with_video(|video| {
        writeln!(video.clear(), "Kernel start...");
    });

    *MEMORY_MAP.lock() = Some(TargetArch::take_memory_map()).expect("No memory map provided by Arch!");
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator::from_memory_map(MEMORY_MAP.lock().as_mut().unwrap()));
    SCHEDULER.lock().set_idle_thread(Thread::new(idle_thread));
    SCHEDULER.lock().add_thread(Thread::new());
    SCHEDULER.lock().start()
}

pub fn idle_thread() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    threading::scheduler::disable_preemption();
    TargetArch::panic(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
