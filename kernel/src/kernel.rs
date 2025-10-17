pub mod display;

use core::fmt::Write;

use crate::arch::Arch;
use display::color::Color;

use crate::drivers::traits::console::InputConsole;
use crate::drivers::traits::console::VideoConsole;
use crate::drivers::traits::timer::Timer;

pub struct Kernel {
    arch: TargetArch,
}

impl Kernel {
    pub fn init(arch: TargetArch) -> Self {
        Self { arch }
    }

    pub fn run(&mut self) -> ! {
        let mut binding = TargetArch::video().lock();
        let video = binding.as_mut().expect("Video driver wasn't init!");
        let _ = video.clear().write_str("Waiting!\n");
        let mut binding = TargetArch::serial().lock();
        let serial = binding.as_mut().expect("Serial driver wasn't init!");
        let binding = TargetArch::timer().lock();
        let timer = binding.as_ref().expect("Timer driver wasn't init!");
        timer.sleep(100);
        writeln!(serial, "{}", timer.get_uptime_ms());
        loop {
            while serial.has_next_byte() {
                write!(video, "{}", serial.read_byte().unwrap() as char);
            }
        }
    }

    pub fn panic(&mut self, info: &PanicInfo) -> ! {
        if let Some(video_console) = TargetArch::video().lock().as_mut() {
            video_console
                .set_bg(Color::red())
                .set_fg(Color::black())
                .clear();
            let _ = writeln!(video_console, "{}", info);
        }
        if let Some(serial_console) = TargetArch::serial().lock().as_mut() {
            let _ = writeln!(serial_console, "{}", info);
        }
        loop {}
    }
}

use crate::TargetArch;
use core::ptr::NonNull;

static mut KERNEL: Option<NonNull<Kernel>> = None;
static mut ARCH: Option<NonNull<TargetArch>> = None;

pub fn kmain(arch: TargetArch) -> ! {
    unsafe {
        ARCH = Some(NonNull::from(&arch));
    }
    let mut kernel = Kernel::init(arch);
    unsafe {
        KERNEL = Some(NonNull::from(&kernel));
    }
    kernel.run()
}

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    unsafe {
        if let Some(mut kernel) = KERNEL {
            kernel.as_mut().panic(info)
        } else if let Some(mut arch) = ARCH {
            arch.as_mut().panic(info)
        } else {
            loop {}
        }
    }
}
