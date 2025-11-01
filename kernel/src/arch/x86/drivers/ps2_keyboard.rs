use crate::drivers::traits::console::InputConsole;
use crate::drivers::traits::console::keyboard::{KeyboardDriver, KeyEvent, Key, LedsState};
use crate::arch::x86::interrupts::{InterruptStackFrame, register_interrupt_handler};
use crate::arch::x86::pic;
use crate::arch::Arch;
use super::ps2::{self, PS2DeviceType};
use alloc::collections::VecDeque;

use phf::phf_map;

const CONNECTION_TIME_TEST_THRESHOLD_MS: u64 = 2000; // 2 seconds
const IRQ_NUM: u8 = 1;
const IRQ_INT_NUM: u8 = 0x21;

const SET_SCANCODE_SET2_COMMAND: u8 = 0xF0;
const SET_SCANCODE_SET2_DATA: u8 = 2;

const SET_TYPEMATIC_RATE_COMMAND: u8 = 0xF3;
const SET_TYPEMATIC_RATE_VALUE: u8 = 0x00; // Fastest

const SET_LEDS_STATE_COMMAND: u8 = 0xED;

const COMMAND_ACK: u8 = 0xFA;
const COMMAND_RESEND: u8 = 0xFE;

const REGULAR_SCANCODES_MAP: phf::Map<u8, Key> = phf_map! {
    0x1C => Key::A,
    0x32 => Key::B,
    0x21 => Key::C,
    0x23 => Key::D,
    0x24 => Key::E,
    0x2B => Key::F,
    0x34 => Key::G,
    0x33 => Key::H,
    0x43 => Key::I,
    0x3B => Key::J,
    0x42 => Key::K,
    0x4B => Key::L,
    0x3A => Key::M,
    0x31 => Key::N,
    0x44 => Key::O,
    0x4D => Key::P,
    0x15 => Key::Q,
    0x2D => Key::R,
    0x1B => Key::S,
    0x2C => Key::T,
    0x3C => Key::U,
    0x2A => Key::V,
    0x1D => Key::W,
    0x22 => Key::X,
    0x35 => Key::Y,
    0x1A => Key::Z,
    0x45 => Key::Num0,
    0x16 => Key::Num1,
    0x1E => Key::Num2,
    0x26 => Key::Num3,
    0x25 => Key::Num4,
    0x2E => Key::Num5,
    0x36 => Key::Num6,
    0x3D => Key::Num7,
    0x3E => Key::Num8,
    0x46 => Key::Num9,
    0x0E => Key::BackTick,
    0x4E => Key::MinusSign,
    0x55 => Key::EqualSign,
    0x5D => Key::Backslash,
    0x66 => Key::Backspace,
    0x29 => Key::Space,
    0x0D => Key::Tab,
    0x58 => Key::CapsLock,
    0x12 => Key::LeftShift,
    0x14 => Key::LeftControl,
    0x11 => Key::LeftAlt,
    0x59 => Key::RightShift,
    0x5E => Key::Enter,
    0x76 => Key::Esc,
    0x05 => Key::F1,
    0x06 => Key::F2,
    0x04 => Key::F3,
    0x0C => Key::F4,
    0x03 => Key::F5,
    0x0B => Key::F6,
    0x83 => Key::F7,
    0x0A => Key::F8,
    0x01 => Key::F9,
    0x09 => Key::F10,
    0x78 => Key::F11,
    0x07 => Key::F12,
    0x7E => Key::Scroll,
    0x54 => Key::OpeningBrackets,
    0x77 => Key::NumLock,
    0x7C => Key::KeypadAsteriks,
    0x7B => Key::KeypadMinusSign,
    0x79 => Key::KeypadPlusSign,
    0x71 => Key::KeypadDot,
    0x70 => Key::Keypad0,
    0x69 => Key::Keypad1,
    0x72 => Key::Keypad2,
    0x7A => Key::Keypad3,
    0x6B => Key::Keypad4,
    0x73 => Key::Keypad5,
    0x74 => Key::Keypad6,
    0x6C => Key::Keypad7,
    0x75 => Key::Keypad8,
    0x7D => Key::Keypad9,
    0x5B => Key::ClosingBrackets,
    0x4C => Key::Semicolon,
    0x52 => Key::Apostrophe,
    0x41 => Key::Comma,
    0x49 => Key::Dot,
    0x4A => Key::Slash,
};

const E0_SCANCODES_MAP: phf::Map<u8, Key> = phf_map! {
    0x1F => Key::LeftWin,
    0x14 => Key::RightControl,
    0x27 => Key::RightWin,
    0x11 => Key::RightAlt,
    0x2F => Key::Apps,
    0x70 => Key::Insert,
    0x6C => Key::Home,
    0x7D => Key::PageUp,
    0x71 => Key::Delete,
    0x69 => Key::End,
    0x7A => Key::PageDown,
    0x75 => Key::UpArrow,
    0x6B => Key::LeftArrow,
    0x72 => Key::DownArrow,
    0x74 => Key::RightArrow,
    0x4A => Key::KeypadSlash,
    0x5A => Key::KeypadEnter,
};

const PRINT_SCREEN_PRESSED_SCANCODE: [u8; 4] = [0xE0, 0x12, 0xE0, 0x7C];
const PRINT_SCREEN_RELEASED_SCANCODE: [u8; 6] = [0xE0, 0xF0, 0x7C, 0xE0, 0xF0, 0x12];
const PAUSE_SCANCODE: [u8; 8] = [0xE1, 0x14, 0x77, 0xE1, 0xF0, 0x14, 0xF0, 0x77];

enum PS2KeyboardState {
    Default,
    WaitForScancodeAfterBreak,
    WaitForScancodeAfterE0Break,
    WaitForScancodeAfterE0,
    WaitForPrintScreenPressedScancode(usize),
    WaitForPrintScreenReleasedScancode(usize),
    WaitForPauseScancode(usize),
}

impl PS2KeyboardState {
    fn to_key_event(&self, scancode: u8) -> Result<KeyEvent, Self> {
        match self {
            Self::Default => {
                if scancode == 0xF0 {
                    Err(Self::WaitForScancodeAfterBreak)
                } else if scancode == 0xE0 {
                    Err(Self::WaitForScancodeAfterE0)
                } else if scancode == PAUSE_SCANCODE[0] {
                    Err(Self::WaitForPauseScancode(0))
                } else {
                    REGULAR_SCANCODES_MAP.get(&scancode).map(|k| KeyEvent::KeyPressed(k.clone())).ok_or_else(|| Self::Default)
                }
            },
            Self::WaitForScancodeAfterBreak => {
                REGULAR_SCANCODES_MAP.get(&scancode).map(|k| KeyEvent::KeyReleased(k.clone())).ok_or_else(|| Self::Default)
            },
            Self::WaitForScancodeAfterE0 => {
                if scancode == 0xF0 {
                    Err(Self::WaitForScancodeAfterE0Break)
                } else if scancode == PRINT_SCREEN_PRESSED_SCANCODE[1] {
                    Err(Self::WaitForPrintScreenPressedScancode(1))
                } else {
                    E0_SCANCODES_MAP.get(&scancode).map(|k| KeyEvent::KeyPressed(k.clone())).ok_or_else(|| Self::Default)
                }
            },
            Self::WaitForScancodeAfterE0Break => {
                if scancode == PRINT_SCREEN_RELEASED_SCANCODE[2] {
                    Err(Self::WaitForPrintScreenReleasedScancode(2))
                } else {
                    E0_SCANCODES_MAP.get(&scancode).map(|k| KeyEvent::KeyReleased(k.clone())).ok_or_else(|| Self::Default)
                }
            },
            Self::WaitForPrintScreenPressedScancode(multibyte_index) => {
                if scancode == PRINT_SCREEN_PRESSED_SCANCODE[multibyte_index + 1] {
                    if multibyte_index + 1 == PRINT_SCREEN_PRESSED_SCANCODE.len() - 1 {
                        Ok(KeyEvent::KeyPressed(Key::PrintScreen))
                    } else {
                        Err(Self::WaitForPrintScreenPressedScancode(multibyte_index + 1))
                    }
                } else {
                    Err(Self::Default)
                }
            },
            Self::WaitForPrintScreenReleasedScancode(multibyte_index) => {
                if scancode == PRINT_SCREEN_RELEASED_SCANCODE[multibyte_index + 1] {
                    if multibyte_index + 1 == PRINT_SCREEN_RELEASED_SCANCODE.len() - 1 {
                        Ok(KeyEvent::KeyReleased(Key::PrintScreen))
                    } else {
                        Err(Self::WaitForPrintScreenReleasedScancode(multibyte_index + 1))
                    }
                } else {
                    Err(Self::Default)
                }
            },
            Self::WaitForPauseScancode(multibyte_index) => {
                if scancode == PAUSE_SCANCODE[multibyte_index + 1] {
                    if multibyte_index + 1 == PAUSE_SCANCODE.len() - 1 {
                        Ok(KeyEvent::KeyPressed(Key::Pause))
                    } else {
                        Err(Self::WaitForPauseScancode(multibyte_index + 1))
                    }
                } else {
                    Err(Self::Default)
                }
            },
        }
    }
}

pub struct PS2Keyboard {
    keyboard_type: PS2DeviceType,
    current_state: PS2KeyboardState,
    current_leds_state: LedsState,
    events_queue: VecDeque<KeyEvent>,
    command_send_queue: VecDeque<(u8, Option<u8>)>,
    waiting_for_command_ack: bool,
    last_connection_time: u64,
}

impl PS2Keyboard {
    pub fn init(keyboard_type: PS2DeviceType) -> Result<Self, ()> {
        if !keyboard_type.is_keyboard() {
            return Err(());
        }
        let mut command_queue = VecDeque::new();
        command_queue.push_back((SET_SCANCODE_SET2_COMMAND, Some(SET_SCANCODE_SET2_DATA)));
        command_queue.push_back((SET_LEDS_STATE_COMMAND, Some(Self::leds_state_to_raw_data(LedsState::default()))));
        command_queue.push_back((SET_TYPEMATIC_RATE_COMMAND, Some(SET_TYPEMATIC_RATE_VALUE)));
        pic::unmask_irq(IRQ_NUM);
        register_interrupt_handler(IRQ_INT_NUM, Self::irq_handler);
        let current_uptime;
        if let Some(arch_drivers) = crate::TargetArch::arch_drivers() {
            current_uptime = arch_drivers.timer.get_uptime_ms();
        } else {
            return Err(());
        }
        Ok(Self {
            events_queue: VecDeque::new(),
            command_send_queue: command_queue,
            current_state: PS2KeyboardState::Default,
            current_leds_state: LedsState::default(),
            waiting_for_command_ack: false,
            keyboard_type,
            last_connection_time: current_uptime,
        })
    }

    fn leds_state_to_raw_data(state: LedsState) -> u8 {
        (state.scroll_lock_on as u8) | ((state.num_lock_on as u8) << 1) | ((state.caps_lock_on as u8) << 2)
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
        let next_scancode = ps2::read_data_with_timeout();
        if let Ok(scancode) = next_scancode {
            if scancode == COMMAND_ACK && self.waiting_for_command_ack {
                let _ = self.command_send_queue.pop_front();
                self.waiting_for_command_ack = false;
            } else if scancode == COMMAND_RESEND && self.waiting_for_command_ack {
                self.waiting_for_command_ack = false; // Will resend the command later
            } else {
                match self.current_state.to_key_event(scancode) {
                    Ok(key_event) => self.events_queue.push_back(key_event),
                    Err(new_state) => self.current_state = new_state,
                };
            }
        }
        if !self.waiting_for_command_ack && let Some(command) = self.command_send_queue.front() {
            let first_result = ps2::send_device_data_port1(command.0).is_ok();
            if let Some(data) = command.1 {
                let second_result = ps2::send_device_data_port1(data).is_ok();
                self.waiting_for_command_ack = first_result && second_result;
            } else {
                self.waiting_for_command_ack = first_result;
            }
        }
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
        self.events_queue.pop_front()
    }

    fn has_next_key(&self) -> bool {
        !self.events_queue.is_empty()
    }

    fn is_connected(&mut self) -> bool {
        pic::mask_irq(IRQ_NUM);
        let current_uptime = crate::TargetArch::arch_drivers().expect("Timer driver is not initialized!").timer.get_uptime_ms();

        let connected = if current_uptime > self.last_connection_time + CONNECTION_TIME_TEST_THRESHOLD_MS {
            ps2::echo_device_port1()
        } else {
            true
        };
        
        if connected {
            self.last_connection_time = current_uptime;
        }
        pic::unmask_irq(IRQ_NUM);
        connected
    }

    fn set_leds_state(&mut self, new_state: LedsState) {
        self.command_send_queue.push_back((SET_LEDS_STATE_COMMAND, Some(Self::leds_state_to_raw_data(new_state))));
        self.current_leds_state = new_state;
    }

    fn get_leds_state(&self) -> LedsState {
        self.current_leds_state
    }
}
