pub mod display;


use crate::arch::Arch;
use display::color::Color;

pub struct Kernel {
    arch: TargetArch,
}

impl Kernel {
    pub fn init(arch: TargetArch) -> Self {
        Self { arch }
    }

    pub fn run(&mut self) -> ! {
        // Access drivers through arch_drivers() - this is safe because we're the only
        // non-interrupt code running, and interrupts don't hold references across calls
        if let Some(arch_drivers) = TargetArch::arch_drivers() {
            if let Some(video) = arch_drivers.video.as_mut() {
                let _ = video.clear().write_str("Kernel start init!\n");
            }
            let timer = arch_drivers.timer.as_mut();
            timer.sleep(100);
            if let Some(video) = arch_drivers.video.as_mut() {
                let _ = writeln!(video, "{:?}", timer.get_uptime_ms());
            }
        }

        loop {
            core::hint::spin_loop();
        }
    }

    pub fn panic(&mut self, info: &PanicInfo) -> ! {
        if let Some(arch_drivers) = TargetArch::arch_drivers() {
            if let Some(video) = arch_drivers.video.as_mut() {
                video.set_bg(Color::red()).set_fg(Color::black()).clear();
                let _ = writeln!(video, "{:?}", info);
            }
            if let Some(serial) = arch_drivers.serial.as_mut() {
                let _ = writeln!(serial, "{:?}", info);
            }
        }

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
