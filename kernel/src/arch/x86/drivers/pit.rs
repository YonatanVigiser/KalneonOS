const DATA_PORT: u16 = 0x40;
const COMMAND_PORT: u16 = 0x43;

const COMMAND: u8 = 0b00110110; // Channel 0, square wave generator, word accessing mode
const RELOAD_VALUE: u16 = 11932; // Each reload = 10ms

const IRQ_INT_NUM: u8 = 0x20;

use crate::arch::x86::cpu::outb;
use crate::drivers::traits::timer::Timer;
use crate::arch::x86::interrupts;
use crate::arch::x86::pic;
use core::sync::atomic::{AtomicU32, Ordering};

static COUNTER_LOW: AtomicU32 = AtomicU32::new(0);
static COUNTER_HIGH: AtomicU32 = AtomicU32::new(0);

pub struct PitTimer();

impl PitTimer {
    pub fn init() -> Self {
        outb(COMMAND_PORT, COMMAND);
        outb(DATA_PORT, RELOAD_VALUE as u8);
        outb(DATA_PORT, (RELOAD_VALUE >> 8) as u8);
        interrupts::register_interrupt_handler(IRQ_INT_NUM, Self::irq);
        COUNTER_LOW.store(0, Ordering::Relaxed);
        COUNTER_HIGH.store(0, Ordering::Relaxed);
        pic::unmask_irq(0);

        Self()
    }

    fn irq(_stack_info: &mut interrupts::InterruptStackFrame) {
        Self::tick();
        pic::send_eoi(0);
    }

    fn tick() {
        COUNTER_LOW.fetch_add(1, Ordering::Relaxed);
        if COUNTER_LOW.load(Ordering::Relaxed) == 0 {
            COUNTER_HIGH.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Timer for PitTimer {
    fn get_uptime_ms(&self) -> u64 {
        ((COUNTER_HIGH.load(Ordering::Relaxed) as u64) << 32 | COUNTER_LOW.load(Ordering::Relaxed) as u64) * 10
    }

    fn sleep(&self, ms: u64) {
        let target_time_ms = self.get_uptime_ms() + ms;
        while self.get_uptime_ms() < target_time_ms {}
    }
}
