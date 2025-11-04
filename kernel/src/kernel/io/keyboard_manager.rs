use crate::drivers::traits::console::keyboard::{Key, KeyEvent, KeyboardDriver};

#[repr(u8)]
#[allow(non_camel_case_types)]
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
    Dot = 46,
    Slash = 47,
    Num0 = 48,
    Num1 = 49,
    Num2 = 50,
    Num3 = 51,
    Num4 = 52,
    Num5 = 53,
    Num6 = 54,
    Num7 = 55,
    Num8 = 56,
    Num9 = 57,
    Colon = 58,
    Semicolon = 59,
    LessThan = 60,
    EqualSign = 61,
    GreaterThan = 62,
    QuestionMark = 63,
    AtSign = 64,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    OpeningBrackets = 91,
    Backslash = 92,
    ClosingBrackets = 93,
    CaretSymbol = 94,
    Underscore = 95,
    Backtick = 96,
    a = 97,
    b = 98,
    c = 99,
    d = 100,
    e = 101,
    f = 102,
    g = 103,
    h = 104,
    i = 105,
    j = 106,
    k = 107,
    l = 108,
    m = 109,
    n = 110,
    o = 111,
    p = 112,
    q = 113,
    r = 114,
    s = 115,
    t = 116,
    u = 117,
    v = 118,
    w = 119,
    x = 120,
    y = 121,
    z = 122,
    OpeningBrace = 123,
    VerticalBar = 124,
    ClosingBrace = 125,
    TildeSign = 126,
    Delete = 127,
}

impl AsciiChar {
    fn from_key(key: Key, shift: bool, caps_lock: bool, num_lock: bool) -> Option<Self> {
        if !Self::is_convertable_to_ascii(key) {
            return None;
        }
        if key.is_number() && key.is_keypad() {
            if num_lock {
                return Self::match_number_key_to_num(key);
            } else {
                return None;
            }
        }
        if key.is_number() && !shift {
            return Self::match_number_key_to_num(key);
        }
        if key.is_number() && shift {
            return Self::match_number_key_to_symbol(key);
        }
        if key.is_alphabetic() {
            return Self::match_alphabetic_key(key, shift ^ caps_lock);
        }
        if key.is_symbol() {
            return Self::match_symbol_key(key, shift);
        }
        None
    }

    fn match_alphabetic_key(key: Key, capitalize: bool) -> Option<Self> {
        if capitalize {
            match key {
                Key::A => Some(AsciiChar::A),
                Key::B => Some(AsciiChar::B),
                Key::C => Some(AsciiChar::C),
                Key::D => Some(AsciiChar::D),
                Key::E => Some(AsciiChar::E),
                Key::F => Some(AsciiChar::F),
                Key::G => Some(AsciiChar::G),
                Key::H => Some(AsciiChar::H),
                Key::I => Some(AsciiChar::I),
                Key::J => Some(AsciiChar::J),
                Key::K => Some(AsciiChar::K),
                Key::L => Some(AsciiChar::L),
                Key::M => Some(AsciiChar::M),
                Key::N => Some(AsciiChar::N),
                Key::O => Some(AsciiChar::O),
                Key::P => Some(AsciiChar::P),
                Key::Q => Some(AsciiChar::Q),
                Key::R => Some(AsciiChar::R),
                Key::S => Some(AsciiChar::S),
                Key::T => Some(AsciiChar::T),
                Key::U => Some(AsciiChar::U),
                Key::V => Some(AsciiChar::V),
                Key::W => Some(AsciiChar::W),
                Key::X => Some(AsciiChar::X),
                Key::Y => Some(AsciiChar::Y),
                Key::Z => Some(AsciiChar::Z),
                _ => None,
            }
        } else {
            match key {
                Key::A => Some(AsciiChar::a),
                Key::B => Some(AsciiChar::b),
                Key::C => Some(AsciiChar::c),
                Key::D => Some(AsciiChar::d),
                Key::E => Some(AsciiChar::e),
                Key::F => Some(AsciiChar::f),
                Key::G => Some(AsciiChar::g),
                Key::H => Some(AsciiChar::h),
                Key::I => Some(AsciiChar::i),
                Key::J => Some(AsciiChar::j),
                Key::K => Some(AsciiChar::k),
                Key::L => Some(AsciiChar::l),
                Key::M => Some(AsciiChar::m),
                Key::N => Some(AsciiChar::n),
                Key::O => Some(AsciiChar::o),
                Key::P => Some(AsciiChar::p),
                Key::Q => Some(AsciiChar::q),
                Key::R => Some(AsciiChar::r),
                Key::S => Some(AsciiChar::s),
                Key::T => Some(AsciiChar::t),
                Key::U => Some(AsciiChar::u),
                Key::V => Some(AsciiChar::v),
                Key::W => Some(AsciiChar::w),
                Key::X => Some(AsciiChar::x),
                Key::Y => Some(AsciiChar::y),
                Key::Z => Some(AsciiChar::z),
                _ => None,
            }
        }
    }

    fn match_number_key_to_num(key: Key) -> Option<Self> {
        match key {
            Key::Num0 | Key::Keypad0 => Some(Self::Num0),
            Key::Num1 | Key::Keypad1 => Some(Self::Num1),
            Key::Num2 | Key::Keypad2 => Some(Self::Num2),
            Key::Num3 | Key::Keypad3 => Some(Self::Num3),
            Key::Num4 | Key::Keypad4 => Some(Self::Num4),
            Key::Num5 | Key::Keypad5 => Some(Self::Num5),
            Key::Num6 | Key::Keypad6 => Some(Self::Num6),
            Key::Num7 | Key::Keypad7 => Some(Self::Num7),
            Key::Num8 | Key::Keypad8 => Some(Self::Num8),
            Key::Num9 | Key::Keypad9 => Some(Self::Num9),
            _ => None,
        }
    }

    fn match_number_key_to_symbol(key: Key) -> Option<Self> {
        match key {
            Key::Num0 => Some(Self::ClosingParenthesis),
            Key::Num1 => Some(Self::ExclamationMark),
            Key::Num2 => Some(Self::AtSign),
            Key::Num3 => Some(Self::NumberSign),
            Key::Num4 => Some(Self::DollarSign),
            Key::Num5 => Some(Self::PercentSign),
            Key::Num6 => Some(Self::CaretSymbol),
            Key::Num7 => Some(Self::Ampersand),
            Key::Num8 => Some(Self::Asterisk),
            Key::Num9 => Some(Self::OpeningParenthesis),
            _ => None,
        }
    }

    fn is

    fn is_convertable_to_ascii(key: Key) -> bool {
        match key {
            Key::CapsLock => false,
            Key::LeftShift => false,
            Key::LeftControl => false,
            Key::LeftWin => false,
            Key::LeftAlt => false,
            Key::RightShift => false,
            Key::RightControl => false,
            Key::RightWin => false,
            Key::RightAlt => false,
            Key::Apps => false,
            Key::Enter => false,
            Key::F1 => false,
            Key::F2 => false,
            Key::F3 => false,
            Key::F4 => false,
            Key::F5 => false,
            Key::F6 => false,
            Key::F7 => false,
            Key::F8 => false,
            Key::F9 => false,
            Key::F10 => false,
            Key::F11 => false,
            Key::F12 => false,
            Key::PrintScreen => false,
            Key::Scroll => false,
            Key::Pause => false,
            Key::Insert => false,
            Key::Home => false,
            Key::PageUp => false,
            Key::Delete => false,
            Key::End => false,
            Key::PageDown => false,
            Key::UpArrow => false,
            Key::LeftArrow => false,
            Key::DownArrow => false,
            Key::RightArrow => false,
            Key::NumLock => false,
            _ => true,
        }
    }
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
