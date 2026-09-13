use core::fmt::Write;

use crate::arch::BOOT_INFO;
use crate::platform::acpi::ACPI;
use acpi::HpetInfo;

use self::display::{DisplayInfo, framebuffer, vga};

pub mod com;
pub mod display;
pub mod input;
pub mod time;

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
    let boot_info = BOOT_INFO.get().unwrap();
    if let Some(display_info) = boot_info.display.as_ref() {
        match display_info {
            DisplayInfo::Graphics(info) => framebuffer::init(info),
            DisplayInfo::Text(info) => vga::init(),
        }
    }
    log::info!("Stage2 Drivers were init!");
}

pub fn init_stage3() {
    input::init();
    log::info!("Stage3 Drivers were init!");
}
