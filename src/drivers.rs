use acpi::HpetInfo;

use crate::platform::acpi::ACPI;

pub mod display;
pub mod time;

pub fn init() {
    let acpi = ACPI.poll().unwrap();
    time::hpet::init(HpetInfo::new(&acpi.tables).expect("No HPET info in ACPI tables!"));
}
