use crate::drivers::traits::console::keyboard::{Key, KeyEvent};
use crate::arch::Arch;
use crate::TargetArch;
use crate::kernel::UPTIME_MS;
use core::sync::atomic::Ordering;

#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum AsciiChar {
    Backspace = 8,
    Tab = 9,
    LineFeed = 10,
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
    BackTick = 96,
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
        if key.is_space() {
            return Self::match_space_key(key);
        }
        None
    }

    fn match_space_key(key: Key) -> Option<Self> {
        match key {
            Key::Enter | Key::KeypadEnter => Some(Self::LineFeed),
            Key::Space => Some(Self::Space),
            Key::Tab => Some(Self::Tab),
            Key::Delete => Some(Self::Delete),
            Key::Backspace => Some(Self::Backspace),
            Key::Esc => Some(Self::Esc),
            _ => None,
        }
    }

    fn match_alphabetic_key(key: Key, capitalize: bool) -> Option<Self> {
        if capitalize {
            match key {
                Key::A => Some(Self::A),
                Key::B => Some(Self::B),
                Key::C => Some(Self::C),
                Key::D => Some(Self::D),
                Key::E => Some(Self::E),
                Key::F => Some(Self::F),
                Key::G => Some(Self::G),
                Key::H => Some(Self::H),
                Key::I => Some(Self::I),
                Key::J => Some(Self::J),
                Key::K => Some(Self::K),
                Key::L => Some(Self::L),
                Key::M => Some(Self::M),
                Key::N => Some(Self::N),
                Key::O => Some(Self::O),
                Key::P => Some(Self::P),
                Key::Q => Some(Self::Q),
                Key::R => Some(Self::R),
                Key::S => Some(Self::S),
                Key::T => Some(Self::T),
                Key::U => Some(Self::U),
                Key::V => Some(Self::V),
                Key::W => Some(Self::W),
                Key::X => Some(Self::X),
                Key::Y => Some(Self::Y),
                Key::Z => Some(Self::Z),
                _ => None,
            }
        } else {
            match key {
                Key::A => Some(Self::a),
                Key::B => Some(Self::b),
                Key::C => Some(Self::c),
                Key::D => Some(Self::d),
                Key::E => Some(Self::e),
                Key::F => Some(Self::f),
                Key::G => Some(Self::g),
                Key::H => Some(Self::h),
                Key::I => Some(Self::i),
                Key::J => Some(Self::j),
                Key::K => Some(Self::k),
                Key::L => Some(Self::l),
                Key::M => Some(Self::m),
                Key::N => Some(Self::n),
                Key::O => Some(Self::o),
                Key::P => Some(Self::p),
                Key::Q => Some(Self::q),
                Key::R => Some(Self::r),
                Key::S => Some(Self::s),
                Key::T => Some(Self::t),
                Key::U => Some(Self::u),
                Key::V => Some(Self::v),
                Key::W => Some(Self::w),
                Key::X => Some(Self::x),
                Key::Y => Some(Self::y),
                Key::Z => Some(Self::z),
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

    fn match_symbol_key(key: Key, shift: bool) -> Option<Self> {
        if shift {
            match key {
                Key::BackTick => Some(Self::TildeSign),
                Key::MinusSign => Some(Self::Underscore),
                Key::EqualSign | Key::KeypadPlusSign => Some(Self::Plus),
                Key::Backslash => Some(Self::VerticalBar),
                Key::Slash => Some(Self::QuestionMark),
                Key::Dot => Some(Self::GreaterThan),
                Key::Comma => Some(Self::LessThan),
                Key::Semicolon => Some(Self::Colon),
                Key::Apostrophe => Some(Self::DoubleQuote),
                Key::OpeningBrackets => Some(Self::OpeningBrace),
                Key::ClosingBrackets => Some(Self::ClosingBrace),
                _ => None,
            }
        } else {
            match key {
                Key::BackTick => Some(Self::BackTick),
                Key::MinusSign => Some(Self::Minus),
                Key::EqualSign => Some(Self::EqualSign),
                Key::Backslash => Some(Self::Backslash),
                Key::Slash => Some(Self::Slash),
                Key::Dot => Some(Self::Dot),
                Key::Comma => Some(Self::Comma),
                Key::Semicolon => Some(Self::Semicolon),
                Key::Apostrophe => Some(Self::SingleQuote),
                Key::OpeningBrackets => Some(Self::OpeningBrackets),
                Key::ClosingBrackets => Some(Self::ClosingBrackets),
                _ => None,
            }
        }
    }

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
    pub key_press_time: Option<u64>,  // Tick count when current key was first pressed
    pub last_repeat_time: Option<u64>, // Tick count when last repeat occurred
}

impl KeyboardState {
    fn update(&mut self, key: Key, is_pressed: bool, current_time: u64) {
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
                // Only reset timing if this is a new key press
                if self.last_ascii_key_pressed != Some(key) {
                    self.last_ascii_key_pressed = Some(key);
                    self.key_press_time = Some(current_time);
                    self.last_repeat_time = None;
                }
            }
        } else if key.is_printable() && self.last_ascii_key_pressed == Some(key) {
            self.last_ascii_key_pressed = None;
            self.key_press_time = None;
            self.last_repeat_time = None;
        }
        self.pressed_keys[key as usize] = is_pressed;
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        self.pressed_keys[key as usize]
    }
}

pub struct KeyboardManager {
    state: KeyboardState,
}

const DRIVER_NOT_INIT: &str = "Keyboard Driver not initialized!";
const REPEAT_RATE_TIME: u64 = 50;
const INITAL_REPEAT_DELAY_TIME: u64 = 500;

impl KeyboardManager {
    pub const fn init() -> Self {
        let state = KeyboardState {
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
            pressed_keys: [false; Key::Count as usize],
            last_ascii_key_pressed: None,
            key_press_time: None,
            last_repeat_time: None,
        };
        Self {
            state,
        }
    }

    pub fn get_state(&self) -> &KeyboardState {
        &self.state
    }

    pub fn update(&mut self) {
        let current_time = UPTIME_MS.load(Ordering::Acquire) as u64;
        while TargetArch::with_keyboard(|keyboard| keyboard.has_next_key()).expect(DRIVER_NOT_INIT) {
            match TargetArch::with_keyboard(|keyboard| keyboard.next_key()).expect(DRIVER_NOT_INIT).expect("Keyboard reports having key but didn't provide one!") {
                KeyEvent::KeyPressed(key) => self.state.update(key, true, current_time),
                KeyEvent::KeyReleased(key) => self.state.update(key, false, current_time),
            };
        }
    }

    pub fn next_ascii(&mut self) -> Option<AsciiChar> {
        let shift = self.state.is_pressed(Key::LeftShift) | self.state.is_pressed(Key::RightShift);
        let key = self.state.last_ascii_key_pressed?;
        let key_press_time = self.state.key_press_time?;

        let current_time = UPTIME_MS.load(Ordering::Acquire) as u64;

        // First press - always return immediately
        if self.state.last_repeat_time.is_none() {
            self.state.last_repeat_time = Some(current_time);
            return AsciiChar::from_key(key, shift, self.state.caps_lock, self.state.num_lock);
        }

        // Check if we've passed the initial delay
        if current_time < key_press_time + INITAL_REPEAT_DELAY_TIME {
            return None;
        }

        // Check if it's time to repeat
        let last_repeat = self.state.last_repeat_time.unwrap_or(key_press_time);
        let time_since_last_repeat = current_time.saturating_sub(last_repeat);

        if time_since_last_repeat >= REPEAT_RATE_TIME {
            self.state.last_repeat_time = Some(current_time);
            return AsciiChar::from_key(key, shift, self.state.caps_lock, self.state.num_lock);
        }

        None
    }
}
