use crate::drivers::vga::{VGA, Vga};
use core::fmt::Write;
use log::{Level, LevelFilter, Log, Metadata, Record};

pub fn init_boot_logger() {
    *VGA.lock() = Some(Vga::init(80, 25));
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
            x86_64::instructions::interrupts::without_interrupts(|| {
                let mut guard = VGA.lock();
                if let Some(vga) = guard.as_mut() {
                    let _ = writeln!(vga, "{} - {}", record.level(), record.args());
                }
            });
        }
    }

    fn flush(&self) {}
}
