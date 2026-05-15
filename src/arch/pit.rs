use x86_64::instructions::port::PortWriteOnly;

const INIT_COMMAND: u8 = 0b00110110;
const RELOAD_VALUE: u16 = 11932;

pub const TIMER_IRQ: u8 = 0;

pub fn init() {
    let mut data_port = PortWriteOnly::<u8>::new(0x40);
    let mut command_port = PortWriteOnly::<u8>::new(0x43);
    unsafe {
        command_port.write(INIT_COMMAND);
        data_port.write(RELOAD_VALUE as u8);
        data_port.write((RELOAD_VALUE >> 8) as u8);
    }
    todo!("PIT: PIC unmask not yet available");
}
