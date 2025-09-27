mod device_manager;
pub mod display;

use device_manager::DeviceManager;
use crate::arch::Arch;

use core::fmt::Write;
use crate::drivers::traits::console::Console;
use display::color::Color;

pub struct Kernel<A: Arch> {
    arch: A,
    device_manager: DeviceManager,
}

impl<A: Arch> Kernel<A> {
    pub fn init(arch: A) -> Self {
        let device_manager = DeviceManager::init(&arch);
        Self {
            arch,
            device_manager,
        }
    }

    pub fn run(&mut self) -> ! {
        writeln!(self.device_manager.console.clear(), "Hello World!");
        loop {}
    }

    pub fn panic(&mut self, info: &PanicInfo) -> ! {
        writeln!(self.device_manager.console.set_bg(Color::red()).set_fg(Color::black()).clear(), "{}", info);
        loop {}
    }
}

use core::ptr::NonNull;
use crate::TargetArch;

static mut KERNEL: Option<NonNull<Kernel<TargetArch>>> = None;

pub fn kmain(arch: TargetArch) -> ! {
    let mut kernel = Kernel::init(arch);
    unsafe { KERNEL = Some(NonNull::from(&kernel)); }
    kernel.run()
}

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    unsafe {
        if let Some(mut kernel) = KERNEL {
            kernel.as_mut().panic(info)
        }
        else {
            loop { }
        }
    }
}
