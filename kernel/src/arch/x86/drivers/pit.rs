const DATA_PORT: u16 = 0x40;
const COMMAND_PORT: u16 = 0x43;

const COMMAND: u8 = 0b00110110; // Channel 0, square wave generator, word accessing mode
const RELOAD_VALUE: u16 = 11932; // Each reload = 10ms
const MS_PER_TICK: u64 = 10;

const IRQ_INT_NUM: u8 = 0x20;
const IRQ_NUM: u8 = 0;

use crate::arch::x86::cpu::outb;
use crate::arch::x86::interrupts;
use crate::arch::x86::pic;

use crate::arch::Arch;
use crate::kernel::threading::scheduler::SCHEDULER;
use crate::kernel::UPTIME_MS;

pub struct PitTimer();

impl PitTimer {
    pub fn init() {
        outb(COMMAND_PORT, COMMAND);
        outb(DATA_PORT, RELOAD_VALUE as u8);
        outb(DATA_PORT, (RELOAD_VALUE >> 8) as u8);
        interrupts::register_interrupt_handler(IRQ_INT_NUM, Self::handle_irq);
    }

    fn handle_irq(_stack_info: &mut interrupts::InterruptStackFrame) {
        *UPTIME_MS.lock() += MS_PER_TICK;
        SCHEDULER.lock().wake_with_time(*UPTIME_MS.lock());
        pic::send_eoi(IRQ_NUM);
    }
}

unsafe impl Sync for PitTimer {}
