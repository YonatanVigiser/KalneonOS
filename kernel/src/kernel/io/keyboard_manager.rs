use crate::drivers::traits::console::keyboard::{Key, KeyEvent, KeyboardDriver};

#[repr(u8)]
pub enum AsciiChar {
    Backspace = 8,
    Tab = 9,
    Esc = 27,
    Space = 32,
    ExclamationMark = 33,
    DoubleQuote = 34,
    NumberSign = 35,
    DollarSign = 36,
    PercentSign = 37,
    Ampersand = 38,
    SingleQuote = 39,
    OpeningParenthesis = 40,
    ClosingParenthesis = 41,
    Asterisk = 42,
    Plus = 43,
    Comma = 44,
    Minus = 45,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyboardState {
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
    pub pressed_keys: [bool; Key::Count as usize],
    pub last_ascii_key_pressed: Option<Key>,
}

impl KeyboardState {
    fn update(&mut self, key: Key, is_pressed: bool) {
        if is_pressed {
            if let Key::CapsLock = key {
                self.caps_lock = true;
            }
            if let Key::NumLock = key {
                self.num_lock = true;
            }
            if let Key::Scroll = key {
                self.scroll_lock = true;
            }
            if key.is_printable() {
                self.last_ascii_key_pressed = Some(key);
            }
        } else if key.is_printable() {
            self.last_ascii_key_pressed = None;
        }
        self.pressed_keys[key as usize] = is_pressed;
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        self.pressed_keys[key as usize]
    }
}

pub struct KeyboardManager {
    driver: &'static mut dyn KeyboardDriver,
    state: KeyboardState,
}

impl KeyboardManager {
    pub fn init(driver: &'static mut dyn KeyboardDriver) -> Self {
        let state = KeyboardState {
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
            pressed_keys: [false; Key::Count as usize],
            last_ascii_key_pressed: None,
        };
        let mut manager = Self {
            driver,
            state,
        };
        manager.update();
        manager
    }

    pub fn get_state(&self) -> &KeyboardState {
        &self.state
    }

    pub fn update(&mut self) {
        while self.driver.has_next_key() {
            match self.driver.next_key().expect("Keyboard reports having key but didn't provide one!") {
                KeyEvent::KeyPressed(key) => self.state.update(key, true),
                KeyEvent::KeyReleased(key) => self.state.update(key, false),
            };
        }
    }

    pub fn next_ascii(&self) -> Option<AsciiChar> {
        let shift = self.state.is_pressed(Key::LeftShift) | self.state.is_pressed(Key::RightShift);
        let caps_lock = self.state.caps_lock;
        let key = self.state.last_ascii_key_pressed?;
        if (shift ^ caps_lock) && key.is_alphabetic() {
            return Some(key as u8 + 65);
        }
        if key.is_alphabetic() {
            return Some(key as u8 + 97);
        }
        if key.is_number() {
            if !key.is_keypad() {
                if shift {
                    return Some(Self::match_num_to_ascii_symbol(key));
                } else {
                    return Some(key as u8 - Key::Num0 as u8 + 48);
                }
            } else if self.state.num_lock {
                return Some(key as u8 - Key::Keypad0 as u8 + 48);
            }
        }
        if key.is_symbol() {
            return Some(Self::match_symbol_to_ascii(key, shift));
        }
        None
    }

    fn match_symbol_to_ascii(key: Key, shift: bool) -> u8 {
        if shift {
            match key {
                Key::BackTick => 126,
                Key::EqualSign => 43,
                Key::MinusSign => 95,
                Key::Backslash => 124,
                Key::OpeningBrackets => 123,
                Key::ClosingBrackets => 125,
                _ => 0
            }
        } else {
            match key {
                Key::Dot | Key::KeypadDot => 46,
                Key::Slash | Key::KeypadSlash => 47,
                Key::KeypadAsteriks => 42,
                Key::KeypadPlusSign => 45,
                _ => 0,
            }
        }
    }

    fn match_num_to_ascii_symbol(key: Key) -> u8 {
        match key {
            Key::Num0 => 41,
            Key::Num1 => 33,
            Key::Num2 => 64,
            Key::Num3 => 35,
            Key::Num4 => 36,
            Key::Num5 => 37,
            Key::Num6 => 94,
            Key::Num7 => 38,
            Key::Num8 => 42,
            Key::Num9 => 40,
            _ => 0,
        }
    }
}
