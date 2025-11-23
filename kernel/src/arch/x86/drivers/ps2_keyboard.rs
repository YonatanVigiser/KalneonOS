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

const SET_SCANCODE_SET2_COMMAND: u8 = 0xF0;
const SET_SCANCODE_SET2_DATA: u8 = 2;

const SET_TYPEMATIC_RATE_COMMAND: u8 = 0xF3;
const SET_TYPEMATIC_RATE_VALUE: u8 = 0x00; // Fastest

const SET_LEDS_STATE_COMMAND: u8 = 0xED;

const COMMAND_ACK: u8 = 0xFA;
const COMMAND_RESEND: u8 = 0xFE;

const MAX_COMMAND_SEND_RETRYS: u8 = 3;

fn regular_scancode_to_key(scancode: u8) -> Option<Key> {
    match scancode {
        0x1C => Some(Key::A),
        0x32 => Some(Key::B),
        0x21 => Some(Key::C),
        0x23 => Some(Key::D),
        0x24 => Some(Key::E),
        0x2B => Some(Key::F),
        0x34 => Some(Key::G),
        0x33 => Some(Key::H),
        0x43 => Some(Key::I),
        0x3B => Some(Key::J),
        0x42 => Some(Key::K),
        0x4B => Some(Key::L),
        0x3A => Some(Key::M),
        0x31 => Some(Key::N),
        0x44 => Some(Key::O),
        0x4D => Some(Key::P),
        0x15 => Some(Key::Q),
        0x2D => Some(Key::R),
        0x1B => Some(Key::S),
        0x2C => Some(Key::T),
        0x3C => Some(Key::U),
        0x2A => Some(Key::V),
        0x1D => Some(Key::W),
        0x22 => Some(Key::X),
        0x35 => Some(Key::Y),
        0x1A => Some(Key::Z),
        0x45 => Some(Key::Num0),
        0x16 => Some(Key::Num1),
        0x1E => Some(Key::Num2),
        0x26 => Some(Key::Num3),
        0x25 => Some(Key::Num4),
        0x2E => Some(Key::Num5),
        0x36 => Some(Key::Num6),
        0x3D => Some(Key::Num7),
        0x3E => Some(Key::Num8),
        0x46 => Some(Key::Num9),
        0x0E => Some(Key::BackTick),
        0x4E => Some(Key::MinusSign),
        0x55 => Some(Key::EqualSign),
        0x5D => Some(Key::Backslash),
        0x66 => Some(Key::Backspace),
        0x29 => Some(Key::Space),
        0x0D => Some(Key::Tab),
        0x58 => Some(Key::CapsLock),
        0x12 => Some(Key::LeftShift),
        0x14 => Some(Key::LeftControl),
        0x11 => Some(Key::LeftAlt),
        0x59 => Some(Key::RightShift),
        0x5A => Some(Key::Enter),
        0x76 => Some(Key::Esc),
        0x05 => Some(Key::F1),
        0x06 => Some(Key::F2),
        0x04 => Some(Key::F3),
        0x0C => Some(Key::F4),
        0x03 => Some(Key::F5),
        0x0B => Some(Key::F6),
        0x83 => Some(Key::F7),
        0x0A => Some(Key::F8),
        0x01 => Some(Key::F9),
        0x09 => Some(Key::F10),
        0x78 => Some(Key::F11),
        0x07 => Some(Key::F12),
        0x7E => Some(Key::Scroll),
        0x54 => Some(Key::OpeningBrackets),
        0x77 => Some(Key::NumLock),
        0x7C => Some(Key::KeypadAsteriks),
        0x7B => Some(Key::KeypadMinusSign),
        0x79 => Some(Key::KeypadPlusSign),
        0x71 => Some(Key::KeypadDot),
        0x70 => Some(Key::Keypad0),
        0x69 => Some(Key::Keypad1),
        0x72 => Some(Key::Keypad2),
        0x7A => Some(Key::Keypad3),
        0x6B => Some(Key::Keypad4),
        0x73 => Some(Key::Keypad5),
        0x74 => Some(Key::Keypad6),
        0x6C => Some(Key::Keypad7),
        0x75 => Some(Key::Keypad8),
        0x7D => Some(Key::Keypad9),
        0x5B => Some(Key::ClosingBrackets),
        0x4C => Some(Key::Semicolon),
        0x52 => Some(Key::Apostrophe),
        0x41 => Some(Key::Comma),
        0x49 => Some(Key::Dot),
        0x4A => Some(Key::Slash),
        _ => None,
    }
}

fn e0_scancode_to_key(scancode: u8) -> Option<Key> {
    match scancode {
        0x1F => Some(Key::LeftWin),
        0x14 => Some(Key::RightControl),
        0x27 => Some(Key::RightWin),
        0x11 => Some(Key::RightAlt),
        0x2F => Some(Key::Apps),
        0x70 => Some(Key::Insert),
        0x6C => Some(Key::Home),
        0x7D => Some(Key::PageUp),
        0x71 => Some(Key::Delete),
        0x69 => Some(Key::End),
        0x7A => Some(Key::PageDown),
        0x75 => Some(Key::UpArrow),
        0x6B => Some(Key::LeftArrow),
        0x72 => Some(Key::DownArrow),
        0x74 => Some(Key::RightArrow),
        0x4A => Some(Key::KeypadSlash),
        0x5A => Some(Key::KeypadEnter),
        _ => None,
    }
}

const PRINT_SCREEN_PRESSED_SCANCODE: [u8; 4] = [0xE0, 0x12, 0xE0, 0x7C];
const PRINT_SCREEN_RELEASED_SCANCODE: [u8; 6] = [0xE0, 0xF0, 0x7C, 0xE0, 0xF0, 0x12];
const PAUSE_SCANCODE: [u8; 8] = [0xE1, 0x14, 0x77, 0xE1, 0xF0, 0x14, 0xF0, 0x77];

#[derive(Debug)]
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
                    regular_scancode_to_key(scancode).map(|k| KeyEvent::KeyPressed(k)).ok_or_else(|| Self::Default)
                }
            },
            Self::WaitForScancodeAfterBreak => {
                regular_scancode_to_key(scancode).map(|k| KeyEvent::KeyReleased(k)).ok_or_else(|| Self::Default)
            },
            Self::WaitForScancodeAfterE0 => {
                if scancode == 0xF0 {
                    Err(Self::WaitForScancodeAfterE0Break)
                } else if scancode == PRINT_SCREEN_PRESSED_SCANCODE[1] {
                    Err(Self::WaitForPrintScreenPressedScancode(1))
                } else {
                    e0_scancode_to_key(scancode).map(|k| KeyEvent::KeyPressed(k)).ok_or_else(|| Self::Default)
                }
            },
            Self::WaitForScancodeAfterE0Break => {
                if scancode == PRINT_SCREEN_RELEASED_SCANCODE[2] {
                    Err(Self::WaitForPrintScreenReleasedScancode(2))
                } else {
                    e0_scancode_to_key(scancode).map(|k| KeyEvent::KeyReleased(k)).ok_or_else(|| Self::Default)
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
    last_connection_time: u64,
}

impl PS2Keyboard {
    pub fn init(keyboard_type: PS2DeviceType) -> Result<Self, ()> {
        if !keyboard_type.is_keyboard() {
            return Err(());
        }

        let mut driver = Self {
            events_queue: VecDeque::new(),
            current_state: PS2KeyboardState::Default,
            current_leds_state: LedsState::default(),
            keyboard_type,
            last_connection_time: 0,
        };

        driver.send_command(SET_SCANCODE_SET2_COMMAND, Some(SET_SCANCODE_SET2_DATA))?;
        driver.send_command(SET_TYPEMATIC_RATE_COMMAND, Some(SET_TYPEMATIC_RATE_VALUE))?;
        driver.send_command(SET_LEDS_STATE_COMMAND, Some(Self::leds_state_to_raw_data(LedsState::default())))?;

        register_interrupt_handler(IRQ_INT_NUM, Self::irq_handler);

        Ok(driver)
    }

    fn leds_state_to_raw_data(state: LedsState) -> u8 {
        (state.scroll_lock_on as u8) | ((state.num_lock_on as u8) << 1) | ((state.caps_lock_on as u8) << 2)
    }

    fn send_command(&mut self, command: u8, data: Option<u8>) -> Result<(), ()> {
        pic::mask_irq(IRQ_NUM);
        for _ in 0..MAX_COMMAND_SEND_RETRYS {
            if ps2::send_device_data_port1(command).is_err() {
                continue; // Retry on send failure
            }
            if let Some(data) = data {
                if ps2::send_device_data_port1(data).is_err() {
                    continue; // Retry on send failure
                }
            }

            loop {
                let response = ps2::read_data_with_timeout();
                if let Ok(response) = response {
                    if response == COMMAND_ACK {
                        pic::unmask_irq(IRQ_NUM);
                        return Ok(());
                    } else if response == COMMAND_RESEND {
                        break;
                    } else {
                        // Got scancode instead
                        match self.current_state.to_key_event(response) {
                            Ok(key_event) => self.events_queue.push_back(key_event),
                            Err(new_state) => self.current_state = new_state,
                        };
                    }
                } else {
                    break;
                }
            }
        }

        pic::unmask_irq(IRQ_NUM);
        Err(())
    }

    fn irq_handler(_stack_frame: &mut InterruptStackFrame) {
        crate::TargetArch::with_keyboard(|keyboard| keyboard.process_input());
        pic::send_eoi(IRQ_NUM);
    }
}

impl InputConsole for PS2Keyboard {
    fn process_input(&mut self) {
        let next_scancode = ps2::read_data_with_timeout();
        if let Ok(scancode) = next_scancode {
            match self.current_state.to_key_event(scancode) {
                Ok(key_event) => {
                    self.events_queue.push_back(key_event);
                    self.current_state = PS2KeyboardState::Default;
                },
                Err(new_state) => self.current_state = new_state,
            };
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
        let current_uptime = crate::TargetArch::with_timer(|timer| timer.get_uptime_ms()).expect("Timer Driver was not initlialized!");

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
        let _ = self.send_command(SET_LEDS_STATE_COMMAND, Some(Self::leds_state_to_raw_data(new_state)));
        self.current_leds_state = new_state;
    }

    fn get_leds_state(&self) -> LedsState {
        self.current_leds_state
    }
}
