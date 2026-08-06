use acpi::HpetInfo;
use acpi::sdt::spcr::Spcr;
use uart_16550::Uart16550Tty;

use crate::platform::acpi::ACPI;

pub mod display;
pub mod time;
pub mod com;

pub fn init() {
    let acpi = ACPI.poll().unwrap();
    time::hpet::init(HpetInfo::new(&acpi.tables).expect("No HPET info in ACPI tables!"));
    com::uart16550::init(acpi.tables.find_table::<Spcr>());
}
