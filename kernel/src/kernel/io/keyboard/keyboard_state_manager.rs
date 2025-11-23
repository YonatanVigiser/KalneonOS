use crate::drivers::traits::console::keyboard::{Key, KeyEvent};
use crate::arch::Arch;
use crate::TargetArch;
use super::super::ascii::AsciiChar;

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
    pub fn init() -> Self {
        let state = KeyboardState {
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
            pressed_keys: [false; Key::Count as usize],
            last_ascii_key_pressed: None,
            key_press_time: None,
            last_repeat_time: None,
        };
        let mut manager = Self {
            state,
        };
        manager.update();
        manager
    }

    pub fn get_state(&self) -> &KeyboardState {
        &self.state
    }

    pub fn update(&mut self) {
        let current_time = TargetArch::with_timer(|timer| timer.get_uptime_ms()).expect("Timer not initiazed!");
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

        let current_time = TargetArch::with_timer(|timer| timer.get_uptime_ms()).expect("Timer not initiazed!");

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
