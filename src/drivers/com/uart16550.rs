use uart_16550::{Config, Uart16550};

pub const COM1_IO_PORT: u16 = 0x3F8;
pub const COM2_IO_PORT: u16 = 0x2F8;

pub fn init() {
    if let Ok(uart1) = Uart16550::new_port(COM1_IO_PORT) {
        uart1.init(Config::default());
        uart1.test_loopback().
    }
}
