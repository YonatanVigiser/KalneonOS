use heapless::spsc::Queue;
use crate::arch::x86::cpu::{inb, outb};
use crate::arch::x86::{interrupts, pic};
use crate::arch::x86::SERIAL_CONSOLE_REF;

const COM1_IO_PORT: u16 = 0x3F8;
const COM2_IO_PORT: u16 = 0x2F8;

pub struct SerialDriver {
    port: u16,
    queue: Queue<u8, 256>,
}

impl SerialDriver {
    pub fn init() -> Option<Self> {
        let mut driver = None;
        if let Some(result) = Self::try_init_port(COM1_IO_PORT) {
            driver = Some(result);
            interrupts::register_interrupt_handler(0x24, Self::handle_irq);
            pic::unmask_irq(4);
        } else if let Some(result) = Self::try_init_port(COM2_IO_PORT) {
            driver = Some(result);
            interrupts::register_interrupt_handler(0x23, Self::handle_irq);
            pic::unmask_irq(3);
        }
        driver
     }

    fn try_init_port(port: u16) -> Option<Self> {
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
        if inb(port + 0) != 0xAE {
            return None;
        }

        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        outb(port + 4, 0x0F);

        // Enable "received data available" interrupt
        outb(port + 1, 0x01);

        Some(Self{ port, queue: Queue::new() })
    }

    fn handle_irq(_stack_info: &mut interrupts::InterruptStackFrame) {
        if let Some(mut serial_console) = unsafe { SERIAL_CONSOLE_REF } {
            unsafe { serial_console.as_mut() }.process_input();
        }
    }

    fn has_next(&self) -> bool {
        inb(self.port + 5) & 1 == 1
    }

    fn ready_to_send(&self) -> bool {
        inb(self.port + 5) & 0x20 != 0
    }

    fn read_next(&self) -> u8 {
        inb(self.port)
    }

    fn write_byte(&mut self, byte: u8) {
        while !self.ready_to_send() { }
        outb(self.port, byte);
    }
}

use crate::drivers::traits::console::{OutputConsole, InputConsole, SerialConsole};

impl core::fmt::Write for SerialDriver {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

impl OutputConsole for SerialDriver {}

impl InputConsole for SerialDriver {
    fn process_input(&mut self) {
        while self.has_next() {
            let _ = self.queue.enqueue(self.read_next());
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        self.queue.dequeue()
    }

    fn has_next_byte(&self) -> bool {
        !self.queue.is_empty()
    }
}

impl SerialConsole for SerialDriver {}
