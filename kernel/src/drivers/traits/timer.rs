pub trait Timer {
    fn init() -> Self;

    fn get_uptime_ms(&self) -> u64;

    fn sleep(&self, ms: u64);
}
