pub trait Timer {
    fn get_uptime_ms(&self) -> u64;
    fn sleep(&self, ms: u64);
}

use crate::arch::x86::drivers::pit::PitTimer;

pub enum TimerImpl {
    Pit(PitTimer),
}

impl Timer for TimerImpl {
    fn get_uptime_ms(&self) -> u64 {
        match self {
            Self::Pit(pit) => pit.get_uptime_ms(),
        }
    }

    fn sleep(&self, ms: u64) {
        match self {
            Self::Pit(pit) => pit.sleep(ms),
        };
    }
}
