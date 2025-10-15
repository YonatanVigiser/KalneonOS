pub mod cpu;
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
static mut VIDEO_DRIVER: Option<Vga> = None;
static mut SERIAL_DRIVER: Option<SerialDriver> = None;
static mut TIMER_DRIVER: Option<PitTimer> = None;

pub struct ArchX86();

impl Arch for ArchX86 {
    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) -> Self {
        // Init CPU intterupts
        idt::init();
        pic::init();

        // Init early drivers
        unsafe {
            VIDEO_DRIVER = Some(Vga::init(80, 25));
            SERIAL_DRIVER = SerialDriver::init();
            TIMER_DRIVER = Some(PitTimer::init());
        }

        // Finish init - enable interrupts
        unsafe { cpu::sti(); }
        Self()
    }

    fn panic(&mut self, info: &PanicInfo) -> ! {
        use crate::kernel::display::color::Color;
        unsafe { cpu::cli(); }

        if let Some(video_console) = Self::video() {
            let video_console = unsafe { video_console.as_mut() };
            video_console.set_bg(Color::red()).set_fg(Color::black()).clear();
            let _ = writeln!(video_console, "{}", info);
        }
        loop { }
    }

    fn video() -> Option<core::ptr::NonNull<dyn VideoConsole>> {
        unsafe { VIDEO_DRIVER.map(|v| NonNull::from(&v as &dyn VideoConsole)) }
    }

    fn serial() -> Option<core::ptr::NonNull<dyn SerialConsole>> {
        unsafe { SERIAL_DRIVER.map(|s| NonNull::from(&s as &dyn SerialConsole)) }
    }

    fn timer() -> core::ptr::NonNull<dyn Timer> {
        unsafe { NonNull::from(&TIMER_DRIVER.expect("Timer driver wasn't initilized before acceced!") as &dyn Timer) }
    }
}
