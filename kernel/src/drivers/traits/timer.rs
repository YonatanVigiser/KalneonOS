pub trait Timer : Sync {
    fn get_uptime_ms(&self) -> u64;
    fn sleep(&self, ms: u64);
    fn tick(&mut self);
}
