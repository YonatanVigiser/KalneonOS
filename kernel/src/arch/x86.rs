mod cpu;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod drivers;

use super::Arch;

use crate::drivers::traits::console::ConsoleImpl;
use crate::drivers::traits::timer::TimerImpl;

use drivers::vga::Vga;
use drivers::pit::PitTimer;

pub struct ArchX86();

impl Arch for ArchX86 {
    fn init(_boot_info_ptr: usize) -> Self {
        Self()
    }

    fn init_console(&self) -> ConsoleImpl {
        ConsoleImpl::Vga(Vga::new(80, 25))
    }

    fn init_timer(&self) -> TimerImpl {
        TimerImpl::Pit(PitTimer::init())
    }
}
