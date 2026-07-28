use crate::{allow, dev::{GLOBAL_REGISTRY, Info, ReadSync}, device_caps, memory::map_mmio_ptr, time::{KernelInstant, TimerResolution}};
use core::{num::NonZero, sync::atomic::{AtomicU64, Ordering}, u32};
use acpi::HpetInfo;
use ez_hpet::{HPET_MMIO_SIZE, Hpet};

pub(super) struct HpetDriver {
    hpet: Hpet<'static>,
    last_tick : AtomicU64,
    supports_u64: bool,
    period_fs: u64,
}

impl HpetDriver {
    pub fn new(hpet_info: HpetInfo) -> Self {
        let ptr = map_mmio_ptr(hpet_info.base_address, HPET_MMIO_SIZE).expect("MMIO mapping failed!");
        let mut hpet = unsafe { Hpet::new(NonZero::new(ptr).unwrap()) };
        hpet.set_enable(false);
        hpet.set_main_counter_value(0);
        hpet.set_enable(true);
        let supports_u64 = hpet.supports_64_bit_mode();
        let period_fs = hpet.main_counter_tick_period() as u64;
        HpetDriver { hpet, last_tick: AtomicU64::new(0), supports_u64, period_fs }
    }

    fn read_ticks(&self) -> u64 {
        if self.supports_u64 {
            self.hpet.main_counter_value()
        } else {
            let current = self.hpet.main_counter_value() & (u32::MAX as u64);
            loop {
                let last = self.last_tick.load(Ordering::Acquire);
                let last_low = last & (u32::MAX as u64);
                let widened = (last & !(u32::MAX as u64)) + current + if current < last_low { 1u64 << 32 } else { 0 };

                match self.last_tick.compare_exchange_weak(last, widened, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => return widened,
                    Err(_) => continue,
                }
            }
        }
    }

    fn uptime_nano(&self) -> u64 {
        (self.read_ticks() as u128 * self.period_fs as u128 / 1_000_000) as u64
    }
}

impl ReadSync<KernelInstant> for HpetDriver {
    fn read_sync(&self) -> Result<KernelInstant, crate::dev::DeviceError> {
        Ok(KernelInstant::from_ticks(self.uptime_nano()))
    }
}

impl Info<TimerResolution> for HpetDriver {
    fn info(&self) -> TimerResolution {
        TimerResolution::from_nanos(self.period_fs / 1_000_000)
    }
}

device_caps!(HpetDriver: dyn ReadSync<KernelInstant>, dyn Info<TimerResolution>);

pub fn init(hpet_info: HpetInfo) {
    let hpet = HpetDriver::new(hpet_info);
    GLOBAL_REGISTRY.lock().register(hpet, allow!(dyn ReadSync<KernelInstant>, dyn Info<TimerResolution>));
}
