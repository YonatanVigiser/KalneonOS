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
use crate::drivers::traits::timer::Timer;
use crate::arch::x86::interrupts;
use crate::arch::x86::pic;
use core::sync::atomic::{AtomicU32, Ordering};

static COUNTER_LOW: AtomicU32 = AtomicU32::new(0);
static COUNTER_HIGH: AtomicU32 = AtomicU32::new(0);

pub struct PitTimer {
    channels: [Channel; 3],
}

impl PitTimer {
    pub fn init() -> Self {
        let channels = [
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
        ];
        interrupts::register_interrupt_handler(0x20, Self::irq);
        COUNTER_LOW.store(0, Ordering::Relaxed);
        COUNTER_HIGH.store(0, Ordering::Relaxed);
        pic::unmask_irq(0);
        Self {
            channels,
        }
    }

    fn irq(_stack_info: &mut interrupts::InterruptStackFrame) {
        Self::tick();
        pic::send_eoi(0);
    }

    fn tick() {
        COUNTER_LOW.fetch_add(1, Ordering::Relaxed);
        if COUNTER_LOW.load(Ordering::Relaxed) == 0 {
            COUNTER_HIGH.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn set_mode(&mut self,
        channel_num: ChannelNum,
        access_mode: AccessMode,
        mode: OperationMode,
    ) -> Result<(), ()> {
        let mut channel = self.channels[channel_num as u8 as usize];
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

    pub fn set_reload_value(&mut self, channel_num: ChannelNum, mut reload_value: u16) -> Result<(), ()> {
        let mut channel = self.channels[channel_num as u8 as usize];
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

    pub fn get_current_count(&self, channel_num: ChannelNum) -> u16 {
        let channel = self.channels[channel_num as u8 as usize];

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
}

impl Timer for PitTimer {
    fn get_uptime_ms(&self) -> u64 {
        ((COUNTER_HIGH.load(Ordering::Relaxed) << 8) as u64 | COUNTER_LOW.load(Ordering::Relaxed) as u64) * 10
    }

    fn sleep(&self, ms: u64) {
        let target_time_ms = self.get_uptime_ms() + ms;
        while self.get_uptime_ms() < target_time_ms {}
    }
}
