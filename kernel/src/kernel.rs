pub mod display;

use crate::arch::Arch;
use display::color::Color;

pub struct Kernel {
    arch: TargetArch,
}

impl Kernel {
    pub fn init(mut arch: TargetArch) -> Self {
        if let Some(video) = arch.video() {
            video.write_str("\nStart kernel init...");
            video.write_str("\nFinish kernel init!");
        }
        Self {
            arch,
        }
    }

    pub fn run(&mut self) -> ! {
        if let Some(video) = self.arch.video() {
            video.write_str("\nKernel entered mainloop!");
        }
        loop {}
    }

    pub fn panic(&mut self, info: &PanicInfo) -> ! {
        if let Some(video_console) = self.arch.video() {
            video_console.set_bg(Color::red()).set_fg(Color::black()).clear();
            let _ = writeln!(video_console, "{}", info);
        }
        if let Some(serial_console) = self.arch.serial() {
            let _ = writeln!(serial_console, "{}", info);
        }
        loop {}
    }
}

use core::ptr::NonNull;
use crate::TargetArch;

static mut KERNEL: Option<NonNull<Kernel>> = None;
static mut ARCH: Option<NonNull<TargetArch>> = None;

pub fn kmain(arch: TargetArch) -> ! {
    unsafe { ARCH = Some(NonNull::from(&arch)); }
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
        else if let Some(mut arch) = ARCH {
            arch.as_mut().panic(info)
        }
        else {
            loop { }
        }
    }
}
