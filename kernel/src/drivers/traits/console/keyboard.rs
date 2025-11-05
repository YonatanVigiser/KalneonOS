use super::InputConsole;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    BackTick,
    MinusSign,
    EqualSign,
    Backslash,
    Backspace,
    Space,
    Tab,
    CapsLock,
    LeftShift,
    LeftControl,
    LeftWin,
    LeftAlt,
    RightShift,
    RightControl,
    RightWin,
    RightAlt,
    Apps,
    Enter,
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    Scroll,
    Pause,
    OpeningBrackets,
    Insert,
    Home,
    PageUp,
    Delete,
    End,
    PageDown,
    UpArrow,
    LeftArrow,
    DownArrow,
    RightArrow,
    NumLock,
    KeypadSlash,
    KeypadAsteriks,
    KeypadMinusSign,
    KeypadPlusSign,
    KeypadEnter,
    KeypadDot,
    Keypad0,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad4,
    Keypad5,
    Keypad6,
    Keypad7,
    Keypad8,
    Keypad9,
    ClosingBrackets,
    Semicolon,
    Apostrophe,
    Comma,
    Dot,
    Slash,
    Count, // This is only a symbol used to count the number of supported keys. Must ALWAYS remain
           // last!
}

impl Key {
    // Returns if part of the normal chars
    pub fn is_alphabetic(&self) -> bool {
        match self {
            Self::A => true,
            Self::B => true,
            Self::C => true,
            Self::D => true,
            Self::E => true,
            Self::F => true,
            Self::G => true,
            Self::H => true,
            Self::I => true,
            Self::J => true,
            Self::K => true,
            Self::L => true,
            Self::M => true,
            Self::N => true,
            Self::O => true,
            Self::P => true,
            Self::Q => true,
            Self::R => true,
            Self::S => true,
            Self::T => true,
            Self::U => true,
            Self::V => true,
            Self::W => true,
            Self::X => true,
            Self::Y => true,
            Self::Z => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Self::Num0 => true,
            Self::Num1 => true,
            Self::Num2 => true,
            Self::Num3 => true,
            Self::Num4 => true,
            Self::Num5 => true,
            Self::Num6 => true,
            Self::Num7 => true,
            Self::Num8 => true,
            Self::Num9 => true,
            Self::Keypad0 => true,
            Self::Keypad1 => true,
            Self::Keypad2 => true,
            Self::Keypad3 => true,
            Self::Keypad4 => true,
            Self::Keypad5 => true,
            Self::Keypad6 => true,
            Self::Keypad7 => true,
            Self::Keypad8 => true,
            Self::Keypad9 => true,
            _ => false,
        }
    }

    pub fn is_keypad(&self) -> bool {
        match self {
            Self::Keypad0 => true,
            Self::Keypad1 => true,
            Self::Keypad2 => true,
            Self::Keypad3 => true,
            Self::Keypad4 => true,
            Self::Keypad5 => true,
            Self::Keypad6 => true,
            Self::Keypad7 => true,
            Self::Keypad8 => true,
            Self::Keypad9 => true,
            Self::KeypadDot => true,
            Self::KeypadSlash => true,
            Self::KeypadEnter => true,
            Self::KeypadAsteriks => true,
            Self::KeypadPlusSign => true,
            Self::KeypadMinusSign => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            Self::KeypadDot => true,
            Self::KeypadSlash => true,
            Self::KeypadAsteriks => true,
            Self::KeypadPlusSign => true,
            Self::KeypadMinusSign => true,
            Self::BackTick => true,
            Self::MinusSign => true,
            Self::EqualSign => true,
            Self::Backslash => true,
            Self::ClosingBrackets => true,
            Self::Semicolon => true,
            Self::Apostrophe => true,
            Self::Comma => true,
            Self::Dot => true,
            Self::Slash => true,
            _ => false,
        }
    }

    pub fn is_space(&self) -> bool {
        match self {
            Self::Enter | Self::KeypadEnter => true,
            Self::Space => true,
            Self::Tab => true,
            Self::Delete => true,
            Self::Backspace => true,
            Self::Esc => true,
            _ => false,
        }
    }

    pub fn is_printable(&self) -> bool {
        self.is_number() || self.is_alphabetic() || self.is_symbol() || self.is_space()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KeyEvent {
    KeyPressed(Key),
    KeyReleased(Key),
}

/* Highest byte - Pressed / Released */
impl From<KeyEvent> for u8 {
    fn from(value: KeyEvent) -> Self {
        match value {
            KeyEvent::KeyPressed(key) => key as u8,
            KeyEvent::KeyReleased(key) => (key as u8) | 0x80, 
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LedsState {
    pub scroll_lock_on: bool,
    pub num_lock_on: bool,
    pub caps_lock_on: bool,
}

impl LedsState {
    pub fn default() -> Self {
        Self {
            scroll_lock_on: false,
            num_lock_on: false,
            caps_lock_on: false,
        }
    }
}

pub trait KeyboardDriver : InputConsole {
    fn next_key(&mut self) -> Option<KeyEvent>;
    fn has_next_key(&self) -> bool;

    fn is_connected(&mut self) -> bool;

    fn get_leds_state(&self) -> LedsState;
    fn set_leds_state(&mut self, new_state: LedsState);
}
