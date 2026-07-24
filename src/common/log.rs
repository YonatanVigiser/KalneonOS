use crate::{dev::{GLOBAL_REGISTRY, Write}, drivers::display::vga::{VGA, Vga}, task::{Task, executor::Executor, yield_now}};
use core::{fmt::Write as WriteTrait, future::poll_fn, str::FromStr, sync::atomic::{AtomicBool, Ordering}};
use crossbeam_queue::ArrayQueue;
use heapless::String;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Log, Metadata, Record};

pub fn init_logger() {
    *VGA.lock() = Some(Vga::init(80, 25));
    log::set_logger(&LOGGER).expect("Logger init failed!");
    log::set_max_level(LevelFilter::Info);
}

pub fn swap_to_async_logging(executor: &Executor) {
    if LOGGER.is_early.swap(false, Ordering::Acquire) {
        executor.spawn(Task::new(LOGGER.log_task()));
    }
}

fn early_log(record: &Record) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut guard = VGA.lock();
        if let Some(vga) = guard.as_mut() {
        let _ = writeln!(vga, "{} - {}", record.level(), record.args());
        }
    });
}

pub struct Logger {
    is_early: AtomicBool,
}

unsafe impl Sync for Logger {}
unsafe impl Send for Logger {}

impl Logger {
    pub const fn new() -> Self {
        Self { is_early: AtomicBool::new(true) }
    }

    async fn log_task(&self) {
        loop {
            while let Some(msg) = LOG_QUEUE.pop() {
                let devices = GLOBAL_REGISTRY.lock().query::<dyn Write<char>>();
                for device in devices {
                    for c in msg.as_str().chars() {
                        poll_fn(|cx| device.write(cx, &mut Some(c))).await.iter();
                    }
                }
            }
            yield_now().await;
        }
    }
}

pub static LOGGER: Logger = Logger::new();

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info && !VGA.is_locked()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if self.is_early.load(Ordering::Relaxed) {
                early_log(record);
            } else {
                let mut message = String::new();
                let _ = writeln!(message, "{} - {}", record.level(), record.args());
                if LOG_QUEUE.force_push(message).is_some() {
                    let _ = LOG_QUEUE.force_push(String::from_str("WARN - Log buffer overflow").unwrap());
                }
            }
        }
    }

    fn flush(&self) {}
}

const MAX_LOG_LEN: usize = 256;
const QUEUE_SIZE: usize = 256;

lazy_static! {
    static ref LOG_QUEUE: ArrayQueue<String<MAX_LOG_LEN>> = ArrayQueue::new(QUEUE_SIZE);
}
