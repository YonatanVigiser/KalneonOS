use fugit::{NanosDurationU64, TimerInstant};

use crate::dev::{GLOBAL_REGISTRY, Info, ReadSync};

pub mod timer;

pub type KernelInstant = TimerInstant<u64, 1_000_000_000>;
pub type KernelDuration = NanosDurationU64;
pub type TimerResolution = KernelDuration;

pub fn uptime() -> KernelInstant {
        let registry = GLOBAL_REGISTRY.lock();
        let dev_ids = registry.find::<dyn ReadSync<KernelInstant>>();
        let devs = dev_ids.iter().map(|id|
            (registry.get::<dyn ReadSync<KernelInstant>>(*id), registry.get::<dyn Info<TimerResolution>>(*id)));
        let min_res_dev = devs.filter(|dev| dev.0.is_some()).min_by_key(|dev| dev.1.as_ref().map(|info_dev| info_dev.info()).unwrap_or(TimerResolution::MAX));
        min_res_dev.expect("No uptime device registered!").0.unwrap().read_sync().expect("Uptime device access failed!")
}

pub fn stall(duration: KernelDuration) {
    let start = uptime();
    while uptime().checked_duration_since(start).unwrap() < duration {
        core::hint::spin_loop();
    }
}
