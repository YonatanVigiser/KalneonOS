use core::fmt::Write;

use acpi::HpetInfo;
use crate::platform::acpi::ACPI;

pub mod display;
pub mod time;
pub mod com;

pub fn init_early() {
    com::uart16550::init();
}

pub fn panic_log_sink() -> Option<impl Write> {
    com::uart16550::emergency_tty()
}

pub fn init() {
    let acpi = ACPI.poll().unwrap();
    time::hpet::init(HpetInfo::new(&acpi.tables).expect("No HPET info in ACPI tables!"));
    display::vga::init();
    log::info!("Driver where init!");
}
