use x86_64::instructions::port::PortWriteOnly;

const INIT_COMMAND: u8 = 0b00110110; // Channel 0, square wave generator, word accessing mode
const RELOAD_VALUE: u16 = 11932; // Each reload = 10ms

pub const TIMER_IRQ: u8 = 0;

pub fn init() {
    let mut data_port = PortWriteOnly::<u8>::new(0x40);
    let mut command_port = PortWriteOnly::<u8>::new(0x43);
    unsafe { 
        command_port.write(INIT_COMMAND);
        data_port.write(RELOAD_VALUE as u8);
        data_port.write((RELOAD_VALUE >> 8) as u8);
    }
    crate::interrupts::pic::unmask(TIMER_IRQ);
}
