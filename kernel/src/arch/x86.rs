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

//use core::ptr::NonNull;
use core::panic::PanicInfo;
use core::mem::MaybeUninit;

// Early boot drivers refrences for IRQ handlers
static mut VIDEO_DRIVER: MaybeUninit<Vga> = MaybeUninit::uninit();
static mut SERIAL_DRIVER: MaybeUninit<SerialDriver> = MaybeUninit::uninit();
static mut TIMER_DRIVER: MaybeUninit<PitTimer> = MaybeUninit::uninit();

pub struct ArchX86();

impl Arch for ArchX86 {
    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) -> Self {
        // Init CPU intterupts
        idt::init();
        pic::init();

        // Init early drivers
        unsafe {
            VIDEO_DRIVER.write(Vga::init(80, 25));
            if let Some(serial_driver) = SerialDriver::init() {
                SERIAL_DRIVER.write(serial_driver);
            }
            TIMER_DRIVER.write(PitTimer::init());
        }

        // Finish init - enable interrupts
        unsafe { cpu::sti(); }
        Self()
    }

    fn panic(&mut self, info: &PanicInfo) -> ! {
        use crate::kernel::display::color::Color;
        unsafe { cpu::cli(); }

        if let Some(video_console) = Self::video() {
            video_console.set_bg(Color::red()).set_fg(Color::black()).clear();
            let _ = writeln!(video_console, "{}", info);
        }
        loop { }
    }

    fn video() -> Option<&'static mut dyn VideoConsole> {
        unsafe { VIDEO_DRIVER }
    }

    fn serial() -> Option<&'static mut dyn SerialConsole> {
        unsafe { SERIAL_DRIVER.as_mut().map(|s| s as &'static mut dyn SerialConsole) }
    } 

    fn timer() -> &'static mut dyn Timer {
        unsafe { TIMER_DRIVER.as_mut().unwrap() as &'static mut dyn Timer }
    }
