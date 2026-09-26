use core::fmt::{Error, Write};
use core::future::poll_fn;
use core::pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll};

use alloc::sync::Arc;
use crossbeam_queue::ArrayQueue;
use futures_util::future::select;
use futures_util::task::AtomicWaker;
use uart_16550::backend::{Backend, PioBackend};
use uart_16550::spec::FIFO_SIZE;
use uart_16550::spec::registers::IER;
use uart_16550::{Config, Uart16550, Uart16550Tty};

use crate::common::log::LogSink;
use crate::common::mutex::debug_mutex::Mutex;
use crate::dev::lease::LeaseCell;
use crate::dev::registry::DEVICE_REGISTRY;
use crate::drivers::input::InputHub;
use crate::interrupt::{GlobalInterruptController, InterruptListener};
use crate::interrupt::apic::isa_irq_to_gsi;
use crate::task::executor::EXECUTOR;
use crate::task::{Task, yield_now};

const COM1_IO_PORT: u16 = 0x3F8;
const COM2_IO_PORT: u16 = 0x2F8;

const COM1_ISA_IRQ: u8 = 0x4;
const COM2_ISA_IRQ: u8 = 0x3;

fn try_init_port(port: u16) -> Result<Uart16550<PioBackend>, ()> {
    let mut uart = unsafe { Uart16550::new_port(port).map_err(|_| ())? };
    let mut config = Config::default();
    config.interrupts = IER::DATA_READY | IER::THR_EMPTY;
    uart.init(config).map_err(|_| ())?;
    uart.test_loopback().map_err(|_| ())?;
    Ok(uart)
}

pub fn init() {
    let global_interrupt_controller = DEVICE_REGISTRY.read().query::<dyn GlobalInterruptController>().get(0).expect("No GlobalInterruptController").clone();
    if let Ok(com1_dev) = try_init_port(COM1_IO_PORT) {
        let source = isa_irq_to_gsi(COM1_ISA_IRQ);
        let target = global_interrupt_controller.allocate_target().unwrap();
        global_interrupt_controller.route(source, target.0).expect("Routing failed");
        global_interrupt_controller.unmask(source).expect("Unmasking failed");
        let port = UartPort::new(com1_dev, target.1.listen());
        DEVICE_REGISTRY.write().register::<InputHub<u8>>(port.receive_queue.clone());
        DEVICE_REGISTRY.write().register::<dyn LogSink>(Arc::new(LeaseCell::new(UartOutput(port))));
    }
    if let Ok(com2_dev) = try_init_port(COM2_IO_PORT) {
        let source = isa_irq_to_gsi(COM2_ISA_IRQ);
        let target = global_interrupt_controller.allocate_target().unwrap();
        global_interrupt_controller.route(source, target.0).expect("Routing failed");
        global_interrupt_controller.unmask(source).expect("Unmasking failed");
        let port = UartPort::new(com2_dev, target.1.listen());
        DEVICE_REGISTRY.write().register::<InputHub<u8>>(port.receive_queue.clone());
        DEVICE_REGISTRY.write().register::<dyn LogSink>(Arc::new(LeaseCell::new(UartOutput(port))));
    }
}

pub fn emergency_tty() -> Option<impl Write> {
    unsafe { Uart16550Tty::new_port(COM1_IO_PORT, Config::default()) }.ok()
}

const SEND_QUEUE_SIZE: usize = 1024 * 2;

pub struct UartPort<B: Backend> {
    dev: Mutex<Uart16550<B>>,
    send_queue: ArrayQueue<u8>,
    send_waker: AtomicWaker,
    send_dropped: AtomicUsize,
    receive_queue: Arc<InputHub<u8>>,
}

impl<B: Backend + 'static> UartPort<B> {
    fn new(dev: Uart16550<B>, listener: InterruptListener) -> Arc<Self> {
        let port = Arc::new(Self {
            dev: Mutex::new(dev),
            send_queue: ArrayQueue::new(SEND_QUEUE_SIZE),
            send_waker: AtomicWaker::new(),
            send_dropped: AtomicUsize::new(0),
            receive_queue: Arc::new(InputHub::new()),
        });
        EXECUTOR.get().expect("No executor").spawn(Task::new(port.clone().driver_task(listener)));
        port
    }

    fn poll_send_pending(&self, cx: &mut Context<'_>) -> Poll<()> {
        if !self.send_queue.is_empty() { return Poll::Ready(()); }
        self.send_waker.register(cx.waker());
        if !self.send_queue.is_empty() { Poll::Ready(()) } else { Poll::Pending }
    }


    async fn driver_task(self: Arc<Self>, mut listener: InterruptListener) {
        const RECIVE_BYTES_YIELD_CAP: usize = 256;
        const SEND_BYTES_YIELD_CAP: usize = 256;
        let mut receive_buff = [0; RECIVE_BYTES_YIELD_CAP];
        let mut send_buff = [0; FIFO_SIZE];
        loop {
            let (receive_count, send_count, fifo_busy) = {
                let mut receive_count = 0;
                let mut send_count = 0;
                let mut controller = self.dev.lock();
                loop {
                    receive_count += controller.receive_bytes(&mut receive_buff[receive_count..]);

                    if !self.send_queue.is_empty() && controller.ready_to_send().is_ok() {
                        let mut n = 0;
                        while n < FIFO_SIZE && let Some(byte) = self.send_queue.pop() {
                            send_buff[n] = byte;
                            n += 1;
                        }
                        send_count += controller.send_bytes(&send_buff[..n]);
                    }
                    if send_count >= SEND_BYTES_YIELD_CAP || receive_count >= RECIVE_BYTES_YIELD_CAP || controller.isr().interrupt_type().is_none() {
                        break;
                    }
                }
                let fifo_busy = controller.ready_to_send().is_err();
                (receive_count, send_count, fifo_busy)
            };
            for &byte in &receive_buff[..receive_count] {
                self.receive_queue.push(byte);
            }
            if send_count >= SEND_BYTES_YIELD_CAP || receive_count >= RECIVE_BYTES_YIELD_CAP {
                yield_now().await;
                continue;
            }
            if fifo_busy {
                listener.wait().await;
            } else {
                select(pin::pin!(listener.wait()), pin::pin!(poll_fn(|cx| self.poll_send_pending(cx)))).await;
            }
        }
    }
}

struct UartOutput<B: Backend>(Arc<UartPort<B>>);

impl<B: Backend> Write for UartOutput<B> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.send_waker.wake();
        for byte in s.bytes() {
            if let Err(_) = self.0.send_queue.push(byte) {
                self.0.send_dropped.fetch_add(1, Ordering::Relaxed);
                return Err(Error);
            }
        }
        Ok(())
    }
}
