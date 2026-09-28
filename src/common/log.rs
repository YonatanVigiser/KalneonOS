use alloc::vec::Vec;
use blinkcast::static_mem::{Receiver, Sender};
use futures_util::task::AtomicWaker;
use spin::Once;
use core::fmt::Write;
use core::future::poll_fn;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};
use heapless::String;
use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::arch::cpu::current_cpu;
use crate::dev::registry::{DEVICE_REGISTRY, DeviceId};
use crate::interrupt::guard::InterruptGuard;
use crate::task::yield_now;
use crate::time::uptime;

use super::mutex::debug_mutex::Mutex;

pub trait LogSink: Write + Send + Sync {}
impl<T: Write + Send + Sync> LogSink for T {}

pub fn init_logger() {
    LOGGER.genesis_receiver.call_once(|| LOGGER.sender.new_receiver());
    log::set_logger(&LOGGER).expect("Logger init failed!");
    log::set_max_level(LevelFilter::Trace);
}

const MAX_LOG_LEN: usize = 512;
const LOGS_RING_SIZE: usize = 256;

type LogRecord = String<MAX_LOG_LEN>;

struct SinkCursor {
    id: DeviceId,
    cursor: Receiver<'static, LogRecord, LOGS_RING_SIZE>,
}

struct LogSinks {
    generation: u64,
    cursors: Vec<SinkCursor>,
}


impl LogSinks {
    fn sync(&mut self) {
        let registry = DEVICE_REGISTRY.read();
        let generation = registry.slot_generation::<dyn LogSink>();
        if generation == self.generation { return }
        self.generation = generation;
        for id in registry.ids_with_role::<dyn LogSink>() {
            if !self.cursors.iter().any(|sink| sink.id == id) {
                self.cursors.push(SinkCursor { id, cursor: LOGGER.new_cursor() });
            }
        }
    }

    fn drain(&mut self) -> bool {
        self.sync();
        let mut done = true;
        for sink in &mut self.cursors {
            let Some(mut dev) = DEVICE_REGISTRY.read().try_acquire::<dyn LogSink>(sink.id) else { continue };
            let mut count = 0;
            while count < LOGS_RING_SIZE && let Some(record) = {
                let _irq = InterruptGuard::new();
                sink.cursor.recv()
            } {
                let _ = dev.write_str(&record);
                let _ = dev.write_char('\n');
                count += 1;
            }
            if count == LOGS_RING_SIZE { done = false }
        }
        done
    }
}

pub struct Logger {
    sender: Sender<LogRecord, LOGS_RING_SIZE>,
    pub genesis_receiver: Once<Receiver<'static, LogRecord, LOGS_RING_SIZE>>,
    sinks: Mutex<LogSinks>,
    pending: AtomicBool,
    pub auto_flush: AtomicBool,
    log_task_waker: AtomicWaker,
}

impl Logger {
    const fn new() -> Self {
        Self {
            sender: Sender::new(),
            genesis_receiver: Once::new(),
            sinks: Mutex::new(LogSinks { generation: 0, cursors: Vec::new() }),
            pending: AtomicBool::new(false),
            auto_flush: AtomicBool::new(true),
            log_task_waker: AtomicWaker::new(),
        }
    }

    pub fn new_cursor(&'static self) -> Receiver<'static, LogRecord, LOGS_RING_SIZE> {
        self.genesis_receiver.get().expect("Logger not initialized").clone()
    }
}

pub static LOGGER: Logger = Logger::new();

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut message = LogRecord::new();
            let _ = write!(message, "[{}: {}] {}: {}",
                record.level(), uptime(), current_cpu().logical_id, record.args());
            self.sender.send(message);
            self.pending.store(true, Ordering::Release);
            if self.auto_flush.load(Ordering::Acquire) {
                self.flush();
            } else {
                self.log_task_waker.wake();
            }
        }
    }
    fn flush(&self) {
        while self.pending.load(Ordering::Acquire) {
            // Never block here, during boot this runs from IRQ context
            let Some(mut sinks) = self.sinks.try_lock() else { return };
            self.pending.swap(false, Ordering::AcqRel);
            if !sinks.drain() {
                self.pending.store(true, Ordering::Release);
                return;
            }
        }
    }
}

impl Logger {
    pub fn poll_new_messages(&self, cx: &Context<'_>) -> Poll<()> {
        if self.pending.load(Ordering::Acquire) {
            return Poll::Ready(());
        }
        self.log_task_waker.register(cx.waker());
        if self.pending.load(Ordering::Acquire) {
            Poll::Ready(())
        } else { Poll::Pending }
    }

    pub async fn wait_for_messages(&self) {
        poll_fn(|cx| self.poll_new_messages(cx)).await
    }

    pub async fn log_task(&self) {
        loop {
            self.wait_for_messages().await;
            self.flush();
            yield_now().await;
        }
    }
}
