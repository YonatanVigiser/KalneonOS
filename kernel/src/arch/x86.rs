mod cpu;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod drivers;

use super::Arch;

use crate::drivers::traits::console::{VideoConsole, SerialConsole};
use crate::drivers::traits::timer::Timer;

use drivers::vga::Vga;
use drivers::serial::SerialDriver;
use drivers::pit::PitTimer;

use core::ptr::NonNull;
use core::panic::PanicInfo;

// Early boot drivers refrences for IRQ handlers
pub static mut VIDEO_CONSOLE_REF: Option<NonNull<dyn VideoConsole>> = None;
pub static mut SERIAL_CONSOLE_REF: Option<NonNull<dyn SerialConsole>> = None;
pub static mut TIMER_REF: Option<NonNull<dyn Timer>> = None;

pub struct ArchX86 {
    video_console: Option<Vga>,
    serial_console: Option<SerialDriver>,
    timer: PitTimer,
}

impl Arch for ArchX86 {
    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) -> Self {
        // Init CPU intterupts
        idt::init();
        pic::init();

        // Init early drivers
        let video_console = Some(Vga::init(80, 25));
        let serial_console = SerialDriver::init();
        let timer = PitTimer::init();

        let mut insteance = Self {
            video_console,
            serial_console,
            timer,
        };

        // Register IRQ's
        unsafe {
            VIDEO_CONSOLE_REF = insteance.video_console.as_mut().map(|video| NonNull::from(video as &mut dyn VideoConsole));
            SERIAL_CONSOLE_REF = insteance.serial_console.as_mut().map(|serial| NonNull::from(serial as &mut dyn SerialConsole));
            TIMER_REF = Some(NonNull::from(&mut insteance.timer as &mut dyn Timer));
        }

        if let Some(video) = insteance.video() {
            video.clear().write_str("Start arch init...");
        }

        // Finish init - enable interrupts
        unsafe { cpu::sti(); }
        if let Some(video) = insteance.video() {
            video.write_str("\nFinish arch init!\n");
        }
        insteance
    }

    fn panic(&mut self, info: &PanicInfo) -> ! {
        use core::fmt::Write;
        use crate::kernel::display::color::Color;

        if let Some(video_console) = &mut self.video_console {
            video_console.set_bg(Color::red()).set_fg(Color::black()).clear();
            let _ = writeln!(video_console, "{}", info);
        }
        loop { }
    }

    fn video<'a>(&'a mut self) -> Option<&'a mut dyn VideoConsole> {
        self.video_console.as_mut().map(move |video| video as &'a mut dyn VideoConsole)
    }

    fn serial<'a>(&'a mut self) -> Option<&'a mut dyn SerialConsole> {
        self.serial_console.as_mut().map(move |serial| serial as &'a mut dyn SerialConsole)
    }

    fn timer<'a>(&'a mut self) -> &'a mut dyn Timer {
        &mut self.timer as &'a mut dyn Timer
    }
}
