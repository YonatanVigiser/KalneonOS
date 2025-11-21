pub mod display;
pub mod memory;
pub mod io;

use crate::arch::Arch;
use display::color::Color;
use io::keyboard_manager::KeyboardManager;
use memory::frame_allocator::FrameAllocator;

pub struct Kernel {
    arch: TargetArch,
    frame_allocator: FrameAllocator,
    keyboard_manager: KeyboardManager,
}

impl Kernel {
    pub fn init(arch: TargetArch) -> Self {
        let mut frame_allocator = FrameAllocator::new(usize::MAX);
        TargetArch::with_video(|video| {
            let _ = writeln!(video.clear(), "Kernel start init!");
            let _ = writeln!(video, "{:?}", frame_allocator.alloc().unwrap().start());
        });
        Self {
            arch,
            frame_allocator,
            keyboard_manager: KeyboardManager::init()
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.periodic();
        }
    }

    fn periodic(&mut self) {
        self.keyboard_manager.update();
        if let Some(next_ascii) = self.keyboard_manager.next_ascii() {
            TargetArch::with_video(|video| write!(video, "{}", next_ascii as u8 as char));
        }
    }

    fn sleep(&self, ms: u64) {
        let target_time = ms + TargetArch::with_timer(|timer| timer.get_uptime_ms()).unwrap(); 
        while TargetArch::with_timer(|timer| timer.get_uptime_ms()).unwrap() < target_time { 
            core::hint::spin_loop();
        }
    }

    pub fn panic(&mut self, _info: &PanicInfo) -> ! {
        loop {
            core::hint::spin_loop();
        }
    }
}

use crate::TargetArch;

pub static mut KERNEL: Option<Kernel> = None;

pub fn kmain(arch: TargetArch) -> ! {
    let kernel = Kernel::init(arch);
    unsafe {
        KERNEL = Some(kernel);
        let ptr = core::ptr::addr_of_mut!(KERNEL);
        (*ptr).as_mut().unwrap().run()
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(KERNEL);
        if let Some(kernel) = (*ptr).as_mut() {
            kernel.panic(info)
        } else {
            TargetArch::panic(info)
        }
    }
}

use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
