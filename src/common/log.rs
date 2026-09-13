use crossbeam_queue::ArrayQueue;
use futures_util::task::AtomicWaker;
use spin::Once;
use core::fmt::Write;
use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::task::{Context, Poll};
use heapless::String;
use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::dev::registry::DEVICE_REGISTRY;
use crate::dev::traits::LogSink;

pub fn init_logger() {
    LOGGER.queue.call_once(|| ArrayQueue::new(LOGS_QUEUE_SIZE));
    log::set_logger(&LOGGER).expect("Logger init failed!");
    log::set_max_level(LevelFilter::Info);
}

const MAX_LOG_LEN: usize = 256;
const LOGS_QUEUE_SIZE: usize = 256;

pub struct Logger {
    queue: Once<ArrayQueue<String<MAX_LOG_LEN>>>,
    pub auto_flush: AtomicBool,
    flushing: AtomicBool,
    log_task_waker: AtomicWaker,
    dropped_logs_count: AtomicUsize,
}

impl Logger {
    const fn new() -> Self {
        Self {
            queue: Once::new(),
            auto_flush: AtomicBool::new(true),
            flushing: AtomicBool::new(false),
            log_task_waker: AtomicWaker::new(),
            dropped_logs_count: AtomicUsize::new(0)
        }
    }
}

pub static LOGGER: Logger = Logger::new();

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut message: String<MAX_LOG_LEN> = String::new();
            let _ = writeln!(message, "{} - {}", record.level(), record.args());
            if self.queue.get().unwrap().force_push(message).is_some() {
                self.dropped_logs_count.fetch_add(1, Ordering::Release);
            }
            if self.auto_flush.load(Ordering::Acquire) {
                self.flush();
            } else {
                self.log_task_waker.wake();
            }
        }
    }

    fn flush(&self) {
        let queue = self.queue.get().unwrap();
        if !queue.is_empty() && self.flushing.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            let log_sinks = DEVICE_REGISTRY.read().try_acquire_all::<dyn LogSink>();
            let has_sinks = !log_sinks.is_empty();
            while has_sinks && let Some(message) = queue.pop() {
                for log_sink in &log_sinks {
                    log_sink.log(&message);
                }
            }
            drop(log_sinks);
            self.flushing.store(false, Ordering::Release);
       }
   }
}

impl Logger {
    pub fn dropped_logs_count(&self) -> usize {
        self.dropped_logs_count.load(Ordering::Acquire)
    }

    pub fn poll_new_messages(&self, cx: &Context<'_>) -> Poll<()> {
        if let Some(queue) = self.queue.get() {
            if queue.len() != 0 {
                return Poll::Ready(());
            }
            self.log_task_waker.register(cx.waker());
            if queue.len() != 0 {
                Poll::Ready(())
            } else { Poll::Pending }
        } else {
            Poll::Pending
        }
    }

    pub async fn wait_for_messages(&self) {
        poll_fn(|cx| self.poll_new_messages(cx)).await
    }

    pub async fn log_task(&self) {
        loop {
            self.wait_for_messages().await;
            self.flush();
        }
    }
}
