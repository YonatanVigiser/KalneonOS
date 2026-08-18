use alloc::sync::Arc;
use spin::Mutex;
use uart_16550::backend::{Backend, PioBackend};
use uart_16550::{Config, Uart16550};

use crate::dev::registry::DEVICE_REGISTRY;
use crate::dev::traits::{CharOut, LogSink};

pub const COM1_IO_PORT: u16 = 0x3F8;
pub const COM2_IO_PORT: u16 = 0x2F8;

fn try_init_port(port: u16) -> Result<Uart16550<PioBackend>, ()> {
    let mut uart = unsafe { Uart16550::new_port(port).map_err(|_| ())? };
    uart.init(Config::default()).map_err(|_| ())?;
    uart.test_loopback().map_err(|_| ())?;
    Ok(uart)
}

pub fn init() {
    if let Ok(com1_dev) = try_init_port(COM1_IO_PORT) {
        let dev = Arc::new(UartDev(Mutex::new(com1_dev)));
        let id = DEVICE_REGISTRY.write().register::<dyn CharOut>(dev.clone());
        DEVICE_REGISTRY.write().add_role::<dyn LogSink>(id, dev);
    }
    if let Ok(com2_dev) = try_init_port(COM2_IO_PORT) {
        let dev = Arc::new(UartDev(Mutex::new(com2_dev)));
        let id = DEVICE_REGISTRY.write().register::<dyn CharOut>(dev.clone());
        DEVICE_REGISTRY.write().add_role::<dyn LogSink>(id, dev);
    }
}

struct UartDev<B: Backend>(Mutex<Uart16550<B>>);

impl<B: Backend> CharOut for UartDev<B> {
    fn out(&self, c: char) {
        let mut buf = [0u8; 4];
        self.0.lock().send_bytes_exact(c.encode_utf8(&mut buf).as_bytes());
    }
}

impl<B: Backend> LogSink for UartDev<B> {
    fn log(&self, msg: &str) {
        self.0.lock().send_bytes_exact(msg.as_bytes());
    }
}
