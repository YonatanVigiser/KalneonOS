use alloc::vec::Vec;
use crossbeam_queue::ArrayQueue;
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
const LOGS_QUEUE_SIZE: usize = 256;

pub struct Logger {
    queue: ArrayQueue<String<MAX_LOG_LEN>>,
}

impl Logger {
    const fn new() -> Self {
        Self { queue: ArrayQueue::new(LOGS_QUEUE_SIZE) }
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
            let mut message: String<MAX_LOG_LEN> = String::new();
            let _ = writeln!(message, "{} - {}", record.level(), record.args());
            for log_sink in &*self.log_sinks.read() {
                log_sink.log(&message.as_str());
            }
        }
    }

    fn flush(&self) {}
}
