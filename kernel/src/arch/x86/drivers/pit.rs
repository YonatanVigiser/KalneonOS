const DATA_PORT: u16 = 0x40;
const COMMAND_PORT: u16 = 0x43;

const COMMAND: u8 = 0b00110110; // Channel 0, square wave generator, word accessing mode
const RELOAD_VALUE: u16 = 11932; // Each reload = 10ms

const IRQ_INT_NUM: u8 = 0x20;
const IRQ_NUM: u8 = 0;

use crate::arch::x86::cpu::outb;
use crate::arch::x86::interrupts;
use crate::arch::x86::pic;
use crate::drivers::traits::timer::Timer;

use crate::arch::Arch;

pub struct PitTimer(u64);

impl PitTimer {
    pub fn init() -> Self {
        outb(COMMAND_PORT, COMMAND);
        outb(DATA_PORT, RELOAD_VALUE as u8);
        outb(DATA_PORT, (RELOAD_VALUE >> 8) as u8);
        interrupts::register_interrupt_handler(IRQ_INT_NUM, Self::handle_irq);
        pic::unmask_irq(IRQ_NUM);

        Self(0)
    }

    fn handle_irq(_stack_info: &mut interrupts::InterruptStackFrame) {
        if let Some(arch_drivers) = crate::TargetArch::arch_drivers() {
            arch_drivers.timer.tick();
        }
        pic::send_eoi(IRQ_NUM);
    }
}

unsafe impl Sync for PitTimer {}

impl Timer for PitTimer {
    fn get_uptime_ms(&self) -> u64 {
        self.0 * 10
    }

    fn tick(&mut self) {
        self.0 += 1;
    }

    fn sleep(&self, ms: u64) {
        let target_time = self.get_uptime_ms() + ms;
        while self.get_uptime_ms() < target_time {}
    }
}
