use core::fmt::Write;

use alloc::sync::Arc;
use uart_16550::backend::{Backend, PioBackend};
use uart_16550::{Config, Uart16550, Uart16550Tty};

use crate::common::log::LogSink;
use crate::dev::lease::LeaseCell;
use crate::dev::registry::DEVICE_REGISTRY;

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
        let dev = Arc::new(LeaseCell::new(UartDev(com1_dev)));
        DEVICE_REGISTRY.write().register::<dyn LogSink>(dev);
    }
    if let Ok(com2_dev) = try_init_port(COM2_IO_PORT) {
        let dev = Arc::new(LeaseCell::new(UartDev(com2_dev)));
        DEVICE_REGISTRY.write().register::<dyn LogSink>(dev);
    }
}

pub fn emergency_tty() -> Option<impl Write> {
    unsafe { Uart16550Tty::new_port(COM1_IO_PORT, Config::default()) }.ok()
}

pub struct UartDev<B: Backend>(Uart16550<B>);

impl<B: Backend> Write for UartDev<B> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Ok(self.0.send_bytes_exact(s.as_bytes()))
    }
}
