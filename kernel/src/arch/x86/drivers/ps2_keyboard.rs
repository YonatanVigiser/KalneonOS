use crate::drivers::traits::console::InputConsole;
use crate::drivers::traits::console::keyboard::{KeyboardDriver, KeyEvent, Key, LedsState};
use crate::arch::x86::interrupts::{InterruptStackFrame, register_interrupt_handler};
use crate::arch::x86::pic;
use crate::arch::Arch;
use super::ps2::{self, PS2DeviceType};
use alloc::collections::VecDeque;

const CONNECTION_TIME_TEST_THRESHOLD_MS: u64 = 2000; // 2 seconds
const IRQ_NUM: u8 = 1;
const IRQ_INT_NUM: u8 = 0x21;


enum PS2KeyboardState {
    Default,
    WaitFor,
}

pub struct PS2Keyboard {
    keyborad_type: PS2DeviceType,
    current_state: PS2KeyboardState,
    current_leds_state: LedsState,
    events_queue: VecDeque<KeyEvent>,
    command_send_queue: VecDeque<u8>,
    last_connection_time: u64,
}

impl PS2Keyboard {
    pub fn init(keyborad_type: PS2DeviceType) -> Result<Self, ()> {
        if !keyborad_type.is_keyboard() {
            return Err(());
        }
        ps2::disable_scanning_both_ports();
        pic::unmask_irq(IRQ_NUM);
        register_interrupt_handler(IRQ_INT_NUM, Self::irq_handler);
        ps2::enable_scanning_both_ports();
        let current_uptime;
        if let Some(arch_drivers) = crate::TargetArch::arch_drivers() {
            current_uptime = arch_drivers.timer.get_uptime_ms();
        } else {
            return Err(());
        }
        Ok(Self {
            events_queue: VecDeque::new(),
            command_send_queue: VecDeque::new(),
            current_state: PS2KeyboardState::Default,
            current_leds_state: LedsState::default(),
            keyborad_type,
            last_connection_time: current_uptime,
        })
    }

    fn irq_handler(_stack_frame: &mut InterruptStackFrame) {
        if let Some(arch_drivers) = crate::TargetArch::arch_drivers() {
            arch_drivers.keyboard.as_mut().expect("Keyboard driver isn't found in Arch Drivers!").process_input();
        }
        pic::send_eoi(IRQ_NUM);
    }
}

impl InputConsole for PS2Keyboard {
    fn process_input(&mut self) {
    }

    fn read_byte(&mut self) -> Option<u8> {
        self.next_key().map(|k| k.into())
    }
    
    fn has_next_byte(&self) -> bool {
        self.has_next_key()
    }
}

impl KeyboardDriver for PS2Keyboard {
    fn next_key(&mut self) -> Option<KeyEvent> {
        None
    }

    fn has_next_key(&self) -> bool {
        false
    }

    fn is_connected(&mut self) -> bool {
        let current_uptime = crate::TargetArch::arch_drivers().expect("Timer driver is not initilized!").timer.get_uptime_ms();

        if current_uptime > self.last_connection_time + CONNECTION_TIME_TEST_THRESHOLD_MS && !ps2::echo_device_port1() {
            return false;
        }
        self.last_connection_time = current_uptime;
        true
    }

    fn set_leds_state(&mut self, new_state: LedsState) {
        self.command_send_queue(

    }

    fn get_leds_state(&self) -> LedsState {
        self.current_leds_state
    }
}
