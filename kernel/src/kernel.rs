mod device_manager;
pub mod display;

use device_manager::DeviceManager;
use crate::arch::Arch;

use crate::drivers::traits::console::VideoConsole;
use crate::drivers::traits::timer::Timer;
use display::color::Color;

use core::fmt::Write;

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
        let _ = writeln!(self.device_manager.video_console.clear(), "Hello World!");
        self.device_manager.timer.sleep(500);
        let _ = writeln!(self.device_manager.video_console, "Current count ms: {}", self.device_manager.timer.get_uptime_ms());
        loop {}
    }

    pub fn panic(&mut self, info: &PanicInfo) -> ! {
        let _ = writeln!(self.device_manager.video_console.set_bg(Color::red()).set_fg(Color::black()).clear(), "{}", info);
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
