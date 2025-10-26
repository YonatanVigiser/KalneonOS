pub trait Timer: Sync + Send {
    fn get_uptime_ms(&self) -> u64;
    fn sleep(&self, ms: u64);
    fn tick(&mut self);
}
