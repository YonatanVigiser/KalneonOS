use crate::acpi::AcpiRuntimeHandler;
use crate::memory::map_mmio_ptr;
use acpi::{AcpiTables, sdt::hpet::HpetInfo};
use core::num::NonZero;
use ez_hpet::{HPET_MMIO_SIZE, Hpet};
use spin::Mutex;

static HPET: Mutex<Option<HpetState>> = Mutex::new(None);

struct HpetState {
    hpet: Hpet<'static>,
    last_low: u32,
    high: u64,
}

impl HpetState {
    fn read_ticks(&mut self) -> u64 {
        let current = self.hpet.main_counter_value() as u32;
        if current < self.last_low {
            self.high += 1u64 << 32;
        }
        self.last_low = current;
        self.high | current as u64
    }
}

pub fn init_hpet(tables: &AcpiTables<AcpiRuntimeHandler>) -> Option<()> {
    let hpet_info = HpetInfo::new(tables).ok()?;
    let ptr = map_mmio_ptr(hpet_info.base_address, HPET_MMIO_SIZE)?;
    let mut hpet = unsafe { Hpet::new(NonZero::new(ptr)?) };
    hpet.set_enable(false);
    hpet.set_main_counter_value(0);
    hpet.set_enable(true);
    *HPET.lock() = Some(HpetState {
        hpet,
        last_low: 0,
        high: 0,
    });
    Some(())
}

pub fn uptime_nano() -> u64 {
    let mut hpet_guard = HPET.lock();
    let state = hpet_guard
        .as_mut()
        .expect("uptime_nano() was called before hpet was init");
    let ticks = state.read_ticks();
    let period_fs = state.hpet.main_counter_tick_period() as u64;
    (ticks as u128 * period_fs as u128 / 1_000_000) as u64
}
