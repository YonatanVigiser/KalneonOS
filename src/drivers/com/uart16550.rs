use alloc::sync::Arc;
use uart_16550::backend::PioBackend;
use uart_16550::{Config, Uart16550, Uart16550Tty};

use crate::dev::registry::DEVICE_REGISTRY;

pub const COM1_IO_PORT: u16 = 0x3F8;
pub const COM2_IO_PORT: u16 = 0x2F8;

fn try_init_port(port: u16) -> Result<Uart16550<PioBackend>, ()> {
    let mut uart = unsafe { Uart16550::new_port(port).map_err(|_| ())? };
    uart.init(Config::default());
    uart.test_loopback().map_err(|_| ())?;
    Ok(uart)
}

pub fn init() {
    if let Ok(com1_dev) = try_init_port(COM1_IO_PORT) {
        DEVICE_REGISTRY.write().register(Arc::new(com1_dev));
    }
    if let Ok(com2_dev) = try_init_port(COM2_IO_PORT) {
        DEVICE_REGISTRY.write().register(Arc::new(com2_dev));
    }
}
