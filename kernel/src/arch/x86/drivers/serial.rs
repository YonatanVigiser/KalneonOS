const COM1_PORT: u16 = 0x3F8;
const COM2_PORT: u16 = 0x2F8;
const COM1_IRQ_INT_NUM: u8 = 0x24;
const COM2_IRQ_INT_NUM: u8 = 0x23;

static mut COM1_USED: bool = true;

pub struct SerialDriver();

impl SerialDriver {
    pub fn init() -> Option<Self> {
        let mut driver = None;
        if let Some(result) = try_init_port(COM1_PORT) {
            driver = result;
            unsafe { COM1_USED = true; }
            interrupts::register_interrupt_handler(COM1_IRQ_INT_NUM, Self::irq);
            pic::unmask_irq(4);
        } else if let Some(result) = try_init_port(COM2_PORT) {
            driver = result;
            unsafe { COM1_USED = false; }
            interrupts::register_interrupt_handler(COM2_IRQ_INT_NUM, Self::irq);
            pic::unmask_irq(3);
        }
        driver
     }

    fn try_init_port(port: u16) -> Result<Self, ()> {
        outb(port + 1, 0x00);    // Disable all interrupts
        outb(port + 3, 0x80);    // Enable DLAB (set baud rate divisor)
        outb(port + 0, 0x03);    // Set divisor to 3 (lo byte) 38400 baud
        outb(port + 1, 0x00);    //                  (hi byte)
        outb(port + 3, 0x03);    // 8 bits, no parity, one stop bit
        outb(port + 2, 0xC7);    // Enable FIFO, clear them, with 14-byte threshold
        outb(port + 4, 0x0B);    // IRQs enabled, RTS/DSR set

        outb(port + 4, 0x1E);    // Set in loopback mode, test the serial chip
        outb(port + 0, 0xAE);    // Test serial chip (send byte 0xAE and check if serial returns same byte)

        // Check if serial is not faulty (i.e: same byte as sent)
        if inb(port + 0) == 0xAE {
            return Err(());
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(port + 4, 0x0F);

        // Enable "received data available" interrupt
        outb(port + 1, 0x01);

        Ok(Self())
    }

    fn irq(_stack_info: &InterruptStackFrame) {

    }
}

use crate::drivers::traits::console::SerialConsoleImpl;

impl SerialConsoleImpl for SerialDriver {
}
