const DATA_PORT: u16 = 0x40;
const COMMAND_PORT: u16 = 0x43;

const COMMAND: u8 = 0b00110110; // Channel 0, square wave generator, word accessing mode
const RELOAD_VALUE: u16 = 11932; // Each reload = 10ms

const IRQ_INT_NUM: u8 = 0x20;

use crate::arch::x86::cpu::outb;
use crate::drivers::traits::timer::Timer;
use crate::arch::x86::interrupts;
use crate::arch::x86::pic;
use crate::arch::x86::{TIMER_REF, VIDEO_CONSOLE_REF};

pub struct PitTimer(u64);

impl PitTimer {
    pub fn init() -> Self {
        outb(COMMAND_PORT, COMMAND);
        outb(DATA_PORT, RELOAD_VALUE as u8);
        outb(DATA_PORT, (RELOAD_VALUE >> 8) as u8);
        interrupts::register_interrupt_handler(IRQ_INT_NUM, Self::handle_irq);
        pic::unmask_irq(0);

        Self(0)
    }

    fn handle_irq(_stack_info: &mut interrupts::InterruptStackFrame) {
        unsafe {
            let debug = 0x1000 as *mut u16;
            debug.write_volatile(debug.read_volatile() + 1);
        }
        let timer = unsafe { TIMER_REF.expect("Timer wasn't initialized!").as_mut() };
        timer.tick();
        pic::send_eoi(0);
    }
}

impl Timer for PitTimer {
    fn get_uptime_ms(&self) -> u64 {
        self.0 * 10
    }

    fn sleep(&self, ms: u64) {
        let target_time_ms = self.get_uptime_ms() + ms;
        while self.get_uptime_ms() < target_time_ms {}
    }

    fn tick(&mut self) {
        self.0 += 1;
    }
}
