use alloc::{vec::Vec, sync::Arc};
use crate::{dev::{registry::DEVICE_REGISTRY, traits::LogSink}};
use core::{cell::UnsafeCell, fmt::Write};
use heapless::String;
use log::{Level, LevelFilter, Log, Metadata, Record};
use spin::RwLock;

pub fn init_logger() {
    log::set_logger(&LOGGER).expect("Logger init failed!");
    log::set_max_level(LevelFilter::Info);
}

const MAX_LOG_LEN: usize = 256;

pub struct Logger {
    log_sinks: RwLock<Vec<Arc<dyn LogSink>>>,
    log_sinks_slot_generation: UnsafeCell<u64>, // log_sinks write ensures safe access
}

impl Logger {
    const fn new() -> Self {
        Self { log_sinks: RwLock::new(Vec::new()), log_sinks_slot_generation: UnsafeCell::new(0) }
    }
}

unsafe impl Sync for Logger {}
unsafe impl Send for Logger {}

pub static LOGGER: Logger = Logger::new();

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let new_gen = DEVICE_REGISTRY.read().slot_generation::<dyn LogSink>();
            if let Some(mut write_guard) = self.log_sinks.try_write() && new_gen != unsafe { *self.log_sinks_slot_generation.get() } {
                *write_guard = Vec::from_iter(DEVICE_REGISTRY.read().query::<dyn LogSink>().iter().map(|(_, dev)| dev.clone()));
                unsafe { *self.log_sinks_slot_generation.get() = new_gen; }
            }
            let mut message: String<MAX_LOG_LEN> = String::new();
            let _ = writeln!(message, "{} - {}", record.level(), record.args());
            for log_sink in &*self.log_sinks.read() {
                log_sink.log(&message.as_str());
            }
        }
    }

    fn flush(&self) {}
}
