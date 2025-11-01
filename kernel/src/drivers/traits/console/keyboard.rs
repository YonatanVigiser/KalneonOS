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
    Backspcae,
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
    keypadSlash,
    KeypadAsteriks,
    KeypadMinusSign,
    KeypadPlusSign,
    KepadEnter,
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
}

#[derive(Clone, Copy, Debug)]
pub enum KeyEvent {
    KeyPressed(Key),
    KeyReleased(Key),
}

/* Highest byte - Pressed / Released */
impl Into<u8> for KeyEvent {
    fn into(self) -> u8 {
        match self {
            Self::KeyPressed(key) => key as u8,
            Self::KeyReleased(key) => (key as u8) | 0x80, 
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
