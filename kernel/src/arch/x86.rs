pub mod cpu;
pub mod drivers;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod heap;

use super::Arch;

use crate::drivers::traits::console::{VideoConsole,SerialConsole};
use crate::drivers::traits::timer::Timer;


use drivers::pit::PitTimer;
use drivers::serial::SerialDriver;
use drivers::vga::Vga;

use core::fmt::Write;
use core::panic::PanicInfo;

use alloc::boxed::Box;


// Early boot drivers references for IRQ handlers
pub static VIDEO_DRIVER: Option<&mut dyn VideoConsole> = None;
pub static SERIAL_DRIVER: Option<&mut dyn SerialConsole> = None;
pub static TIMER_DRIVER: Option<&mut dyn Timer> = None;

pub struct ArchX86();

impl Arch for ArchX86 {
    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) -> Self {
        // Init CPU
        idt::init();

        // Init heap
        heap::init_heap();

        // Init intterupts:
        pic::init();

        // Init early drivers
        *VIDEO_DRIVER.lock() = Some(Vga::init(80, 25));
        *SERIAL_DRIVER.lock() = SerialDriver::init();
        *TIMER_DRIVER.lock() = Some(PitTimer::init());

        // Finish init - enable interrupts
        unsafe {
            cpu::sti();
        }
        Self()
    }

    fn panic(&mut self, info: &PanicInfo) -> ! {
        use crate::kernel::display::color::Color;
        unsafe {
            cpu::cli();
        }

        if let Some(ref mut video_console) = *Self::video().lock() {
            video_console
                .set_bg(Color::red())
                .set_fg(Color::black())
                .clear();
            let _ = writeln!(video_console, "{}", info);
        }
        loop {}
    }

    fn video() -> Option<Box<dyn VideoConsole>> {
    }

    fn serial() -> Option<Box<dyn SerialConsole>> {
    }

    fn timer() -> Box<dyn Timer> {
        &TIMER_DRIVER
    }
}
