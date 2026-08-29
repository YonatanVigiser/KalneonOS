use core::fmt::Write;

use acpi::HpetInfo;
use crate::platform::acpi::ACPI;

pub mod display;
pub mod time;
pub mod com;
pub mod input;

pub fn init_stage1() {
    com::uart16550::init();
    log::info!("Stage1 Drivers were init!");
}

pub fn panic_log_sink() -> Option<impl Write> {
    com::uart16550::emergency_tty()
}

pub fn init_stage2() {
    let acpi = ACPI.poll().unwrap();
    time::hpet::init(HpetInfo::new(&acpi.tables).expect("No HPET info in ACPI tables!"));
    display::vga::init();
    log::info!("Stage2 Drivers were init!");
}

pub fn init_stage3() {
    input::init();
    log::info!("Stage3 Drivers were init!");
}
