pub mod display;
pub mod memory;
pub mod io;
pub mod threading;
pub mod sync;

use core::panic::PanicInfo;
use crate::arch::Arch;
use io::keyboard_manager::KeyboardManager;
use memory::map::MemoryMap;
use memory::frame_allocator::FrameAllocator;
use linked_list_allocator::LockedHeap;
use threading::scheduler::SCHEDULER;
use threading::thread::Thread;
use sync::Shared;
use core::sync::atomic::AtomicUsize;

pub static FRAME_ALLOCATOR: Shared<Option<FrameAllocator>> = Shared::new(None);

pub static KEYBOARD_STATE_MANAGER: Shared<KeyboardManager> = Shared::new(KeyboardManager::init());

pub static MEMORY_MAP: Shared<Option<MemoryMap>> = Shared::new(None);

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static UPTIME_MS: AtomicUsize = AtomicUsize::new(0);

use crate::TargetArch;

pub fn kmain() -> ! {
    TargetArch::with_video(|video| {
        writeln!(video.clear(), "Kernel start...");
    });

    *MEMORY_MAP.lock() = Some(TargetArch::take_memory_map()).expect("No memory map provided by Arch!");
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator::from_memory_map(MEMORY_MAP.lock().as_mut().unwrap()));
    SCHEDULER.lock().set_idle_thread(Thread::new(idle_thread));
    SCHEDULER.lock().add_thread(Thread::new(keyboard_thread));
    //SCHEDULER.lock().add_thread(Thread::new(panic_thread));
    SCHEDULER.lock().add_thread(Thread::new(print_1));
    SCHEDULER.lock().add_thread(Thread::new(print_2));
    SCHEDULER.lock().start()
}

pub fn keyboard_thread() -> ! {
    loop {
        SCHEDULER.lock().block(threading::thread::BlockingEvent::Keyboard);
        KEYBOARD_STATE_MANAGER.lock().update();
    }
}

use threading::scheduler;
pub fn print_1() -> ! {
    loop {
        let value = scheduler::preemption_enabled();
        TargetArch::with_video(|v| writeln!(v, "1:{}", value));
        //SCHEDULER.lock().yield_now()
    }
}

pub fn print_2() -> ! {
    loop {
        let value = scheduler::preemption_enabled();
        TargetArch::with_video(|v| writeln!(v, "2:{}", value));
        //SCHEDULER.lock().yield_now()
    }
}

pub fn panic_thread() -> ! {
    panic!("PANIC!!!")
}

pub fn idle_thread() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    threading::scheduler::disable_preemption();
    /*
    if SCHEDULER.lock().is_started() {
        // TODO: Add better logging!
        TargetArch::with_video(|v| writeln!(v, "Thread {}, panicked! Panic info:\n{}", SCHEDULER.lock().current_thread_id(), info));
        SCHEDULER.lock().exit_thread()
    } else {
        TargetArch::panic(info)
    }
    */
    TargetArch::panic(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
