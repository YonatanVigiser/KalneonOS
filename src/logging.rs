use log::{Log, Metadata, Record, LevelFilter, Level};
use core::fmt::Write;
use crate::drivers::vga::VGA;

pub fn init_boot_logger() {
    log::set_logger(&BOOT_LOGGER).expect("Logger init failed!");
    log::set_max_level(LevelFilter::Info);
}

pub struct BootLogger;

static BOOT_LOGGER: BootLogger = BootLogger;

impl Log for BootLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info && !VGA.is_locked()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut guard = VGA.lock();
            if let Some(vga) = guard.as_mut() {
                let _ = writeln!(vga, "{} - {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {
    }
}
