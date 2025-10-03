mod cpu;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod drivers;

use super::Arch;

use crate::drivers::traits::console::{VideoConsoleImpl, SerialConsoleImpl};
use crate::drivers::traits::timer::TimerImpl;

use drivers::vga::Vga;
use drivers::serial::SerialDriver;
use drivers::pit::PitTimer;

pub struct ArchX86();

impl Arch for ArchX86 {
    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) -> Self {
        idt::init();
        pic::init();
        unsafe { cpu::sti(); }
        Self()
    }

    fn init_video_console(&self) -> VideoConsoleImpl {
        VideoConsoleImpl::Vga(Vga::new(80, 25))
    }

    fn init_serial_console(&self) -> SerialConsoleImpl {
        SerialConsoleImpl::X86(SerialDriver::init())
    }

    fn init_timer(&self) -> TimerImpl {
        TimerImpl::Pit(PitTimer::init())
    }
}
