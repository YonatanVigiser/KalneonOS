use core::sync::atomic::{Ordering, AtomicU32};
use crate::drivers::pit::{self, AccessMode, OperationMode, ChannelNum::*};

static TICK_COUNTER_LOW: AtomicU32 = AtomicU32::new(0);
static TICK_COUNTER_HIGH: AtomicU32 = AtomicU32::new(0); 

pub fn init() {
    pit::set_mode(C0, AccessMode::Word, OperationMode::SquareWaveGenerator).unwrap();
    pit::set_reload_value(C0, 11932).unwrap();
}

pub fn get_uptime_ticks() -> u64 {
    loop {
        let high1 = TICK_COUNTER_HIGH.load(Ordering::Relaxed);
        let low   = TICK_COUNTER_LOW.load(Ordering::Relaxed);
        let high2 = TICK_COUNTER_HIGH.load(Ordering::Relaxed);

        if high1 == high2 {
            return ((high1 as u64) << 32) | (low as u64);
        }
    }
}

pub fn get_uptime_ms() -> u64 {
    get_uptime_ticks() * 10
}

pub fn reset() {
    TICK_COUNTER_LOW.store(0, Ordering::Relaxed);
    TICK_COUNTER_HIGH.store(0, Ordering::Relaxed);
}

pub fn tick() {
    let old = TICK_COUNTER_LOW.fetch_add(1, Ordering::Relaxed);
    if old == u32::MAX {
        TICK_COUNTER_HIGH.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn sleep(ms: u64) {
    let inital_time_ms = get_uptime_ms();
    while get_uptime_ms() < inital_time_ms + ms { };
}
