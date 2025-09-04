#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ChannelNum {
    C0 = 0,
    C1 = 1,
    C2 = 2,
}

impl ChannelNum {
    pub fn get_data_port(&self) -> u16 {
        (self.clone() as u8) as u16 + 0x40
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AccessMode {
    LowByte = 1,
    HighByte = 2,
    Word = 3,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OperationMode {
    InterruptOnTerminalCount = 0,
    HardwareRetriggerableOneshot = 1,
    RateGenerator = 2,
    SquareWaveGenerator = 3,
    SoftwareTriggeredStrobe = 4,
    HardwareTriggeredStrobe = 5,
}

#[derive(Clone, Copy)]
pub struct Channel {
    num: ChannelNum,
    reload_value: u16,
    access_mode: AccessMode,
    mode: OperationMode,
}

const COMMAND_PORT: u16 = 0x43;

use AccessMode::*;
use ChannelNum::*;
use OperationMode::*;

use crate::arch::x86::cpu::{inb, outb};
use crate::kernel::timer;

use spin::Mutex;

pub static CHANNELS: Mutex<[Channel; 3]> = Mutex::new([
    Channel {
        num: C0,
        reload_value: 0,
        access_mode: Word,
        mode: SquareWaveGenerator,
    },
    Channel {
        num: C1,
        reload_value: 0,
        access_mode: Word,
        mode: SquareWaveGenerator,
    },
    Channel {
        num: C2,
        reload_value: 0,
        access_mode: Word,
        mode: SquareWaveGenerator,
    },
]);

pub fn set_mode(
    channel_num: ChannelNum,
    access_mode: AccessMode,
    mode: OperationMode,
) -> Result<(), ()> {
    let channel = &mut CHANNELS.lock()[channel_num as u8 as usize];
    if let C0 | C1 = channel_num
        && let HardwareRetriggerableOneshot | HardwareTriggeredStrobe = mode
    {
        return Err(());
    }
    let command: u8 = ((channel.num as u8) << 6) | ((access_mode as u8) << 4) | ((mode as u8) << 1);
    channel.access_mode = access_mode;
    channel.mode = mode;

    outb(COMMAND_PORT, command);
    Ok(())
}

pub fn set_reload_value(channel_num: ChannelNum, mut reload_value: u16) -> Result<(), ()> {
    let channel = &mut CHANNELS.lock()[channel_num as u8 as usize];
    if let RateGenerator | SquareWaveGenerator = channel.mode
        && reload_value == 1
    {
        return Err(());
    }
    if let SquareWaveGenerator = channel.mode {
        reload_value &= 0xFFFE;
    }
    match channel.access_mode {
        LowByte => {
            outb(channel.num.get_data_port(), reload_value as u8);
            channel.reload_value = reload_value & 0x00FF;
        }
        HighByte => {
            outb(channel.num.get_data_port(), (reload_value >> 8) as u8);
            channel.reload_value = reload_value & 0xFF00;
        }
        Word => {
            outb(channel.num.get_data_port(), reload_value as u8);
            outb(channel.num.get_data_port(), (reload_value >> 8) as u8);
            channel.reload_value = reload_value;
        }
    };
    Ok(())
}

pub fn get_count(channel_num: ChannelNum) -> u16 {
    let channel = &CHANNELS.lock()[channel_num as u8 as usize];

    match channel.access_mode {
        LowByte => inb(channel.num.get_data_port()) as u16,
        HighByte => (inb(channel.num.get_data_port()) as u16) << 8,
        Word => {
            let latch_command = (channel_num as u8) << 6;
            outb(COMMAND_PORT, latch_command);
            let mut count: u16 = 0;
            count += inb(channel.num.get_data_port()) as u16;
            count += (inb(channel.num.get_data_port()) as u16) << 8;
            count
        }
    }
}

pub fn get_command(channel_num: ChannelNum) -> u8 {
    let channel = &CHANNELS.lock()[channel_num as u8 as usize];

    let mut read_back_command = 0xE0;
    if let C0 = channel_num {
        read_back_command |= 0x02;
    } else if let C1 = channel_num {
        read_back_command |= 0x04;
    } else {
        read_back_command |= 0x08;
    }
    outb(COMMAND_PORT, read_back_command);
    
    inb(channel_num.get_data_port())
}

pub fn hardware_interrupt() {
    timer::tick();
}
