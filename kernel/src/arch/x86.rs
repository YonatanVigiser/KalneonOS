pub mod cpu;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod drivers;

use super::Arch;

pub struct ArchX86();

impl Arch for ArchX86 {
    type CPU = cpu::CPUController;
    type IntteruptsController = interrupts::IntteruptsController;

    type Console = drivers::vga::Vga;
    type Timer = drivers::pit::Pit;

    pub fn init() -> Self {
    }
}
