use alloc::sync::Arc;
use fugit::{NanosDurationU64, TimerInstant};
use spin::Once;

use crate::dev::registry::DEVICE_REGISTRY;

pub mod timer;

pub type KernelInstant = TimerInstant<u64, 1_000_000_000>;
pub type KernelDuration = NanosDurationU64;
pub type TimerResolution = KernelDuration;

pub trait UptimeSource: Send + Sync {
    fn uptime(&self) -> KernelInstant;
    fn resolution(&self) -> TimerResolution;
}

static UPTIME_DEV: Once<Arc<dyn UptimeSource>> = Once::new();

pub fn init() {
    if let Some(dev) = DEVICE_REGISTRY.read().query::<dyn UptimeSource>().iter().min_by_key(|dev| dev.resolution()) {
        UPTIME_DEV.call_once(|| dev.clone());
    } else {
        panic!("No uptime device!")
    }
}

pub fn uptime() -> KernelInstant {
    if let Some(dev) = UPTIME_DEV.get() {
        dev.uptime()
    } else {
        KernelInstant::from_ticks(0)
    }
}

pub fn stall(duration: KernelDuration) {
    let start = uptime();
    while uptime().checked_duration_since(start).unwrap() < duration {
        core::hint::spin_loop();
    }
}
