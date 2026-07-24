use fugit::{NanosDurationU64, TimerInstant};

use crate::dev::{GLOBAL_REGISTRY, Info, ReadSync};

pub mod timer;

pub type KernelInstant = TimerInstant<u64, 1_000_000_000>;
pub type KernelDuration = NanosDurationU64;
pub type TimerResolution = KernelDuration;

pub fn uptime() -> KernelInstant {
        let uptime_devs_ids = GLOBAL_REGISTRY.lock().find::<dyn ReadSync<KernelInstant>>();
        for uptime_dev_id in uptime_devs_ids {
            if let Some(dev_id) = GLOBAL_REGISTRY.lock().get::<dyn Info<KernelDuration>>(uptime_dev_id) {
            }
        }
}

pub fn stall(duration: KernelDuration) {
    let start = uptime();
    while uptime().checked_duration_since(start).unwrap() < duration {
        core::hint::spin_loop();
    }
}
