pub mod display;
pub mod io;

use crate::arch::Arch;
use display::color::Color;
use io::keyboard_manager::KeyboardManager;

pub struct Kernel {
    arch: TargetArch,
    keyboard_manager: KeyboardManager,
}

impl Kernel {
    pub fn init(arch: TargetArch) -> Self {
        Self { arch, keyboard_manager: KeyboardManager::init() }
    }

    pub fn run(&mut self) -> ! {
        // Access drivers through arch_drivers() - this is safe because we're the only
        // non-interrupt code running, and interrupts don't hold references across calls
        TargetArch::with_video(|video| {
            let _ = video.clear().write_str("Kernel start init!\n");
        });

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
