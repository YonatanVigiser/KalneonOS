pub mod display;

use crate::arch::Arch;
use display::color::Color;

pub struct Kernel {
    arch: TargetArch,
}

impl Kernel {
    pub fn init(mut arch: TargetArch) -> Self {
        Self {
            arch,
        }
    }

    pub fn run(&mut self) -> ! {
        let video = unsafe { TargetArch::video().expect("Video driver wasn't init!").as_mut() };
        let serial = unsafe { TargetArch::serial().expect("Serial driver wasn't init!").as_mut() };
        let timer = unsafe { TargetArch::timer().as_mut() };
        video.clear();
        timer.sleep(100);
        writeln!(video, "{}", timer.get_uptime_ms());
        loop {
            if serial.has_next_byte() {
                write!(video, "hey!");
            }
            while serial.has_next_byte() {
                write!(video, "{}", serial.read_byte().unwrap() as char);
            }
        }
    }

    pub fn panic(&mut self, info: &PanicInfo) -> ! {
        if let Some(video_console) = TargetArch::video(){ 
            let video_console = unsafe { video_console.as_mut() };
            video_console.set_bg(Color::red()).set_fg(Color::black()).clear();
            let _ = writeln!(video_console, "{}", info);
        }
        if let Some(serial_console) = TargetArch::serial() {
            let serial_console = unsafe { serial_console.as_mut() };
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
