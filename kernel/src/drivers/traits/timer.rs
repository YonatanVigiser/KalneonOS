#[enum_dispatch::enum_dispatch]
pub trait Timer {
    fn get_uptime_ms(&self) -> u64;
    fn sleep(&self, ms: u64);
}

#[enum_dispatch::enum_dispatch(Timer)]
pub enum TimerImpl {
    Pit(crate::arch::x86::drivers::pit::PitTimer),
}
