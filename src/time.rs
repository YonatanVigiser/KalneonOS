use alloc::sync::Arc;
use crate::dev::{registry::DEVICE_REGISTRY, traits::UptimeSource};
use fugit::{NanosDurationU64, TimerInstant};
use lazy_static::lazy_static;

pub mod timer;

pub type KernelInstant = TimerInstant<u64, 1_000_000_000>;
pub type KernelDuration = NanosDurationU64;
pub type TimerResolution = KernelDuration;

pub fn uptime() -> KernelInstant {
    lazy_static! {
        static ref UPTIME_DEVICE: Arc<dyn UptimeSource> = {
            let registry = DEVICE_REGISTRY.read();
            let devs = registry.query::<dyn UptimeSource>();
            devs.iter().map(|(_, dev)| dev).min_by_key(|dev| dev.resolution()).expect("No uptime source registered!").clone()
        };
    }
    UPTIME_DEVICE.uptime()
}

pub fn stall(duration: KernelDuration) {
    let start = uptime();
    while uptime().checked_duration_since(start).unwrap() < duration {
        core::hint::spin_loop();
    }
}
