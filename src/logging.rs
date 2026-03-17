use log::{Log, Metadata, Record, LevelFilter};
use core::fmt::Write;
use crate::drivers::vga::VGA;

pub fn init_boot_logger() {
    log::set_logger(&BOOT_LOGGER).expect("Logger init failed!");
    log::set_max_level(LevelFilter::Info);
}

pub struct BootLogger;

static BOOT_LOGGER: BootLogger = BootLogger;

impl Log for BootLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Some(vga) = VGA.lock().as_mut() {
                writeln!(vga, "{} - {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {
    }
}
