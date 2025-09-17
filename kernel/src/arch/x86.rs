pub mod cpu;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod drivers;

use super::Arch;

pub struct ArchX86();

impl Arch for ArchX86 {
    type CPUController = cpu::Controller;
    type IntteruptsController = interrupts::Controller;

    type Console = drivers::vga::Vga;
    type Timer = drivers::pit::Pit;

    fn init() -> Self {}
}
