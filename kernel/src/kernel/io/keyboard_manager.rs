use crate::drivers::traits::console::keyboard::{Key, KeyEvent, KeyboardDriver};

#[derive(Debug, Clone, Copy)]
pub struct KeyboardState {
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
    pub pressed_keys: [bool; Key::Count as usize],
    pub last_key_pressed: Option<Key>,
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
            self.last_key_pressed = Some(key);
        } else {
            self.last_key_pressed = None;
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
            last_key_pressed: None,
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
            match self.driver.next_key() {
                KeyEvent::KeyPressed(key) => self.state.update(key, true),
                KeyEvent::KeyReleased(key) => self.state.update(key, false),
            };
        }
    }

    pub fn next_ascii(&self) -> Option<u8> {
        let shift = self.state.is_pressed(Key::LeftShift) | self.state.is_pressed(Key::RightShift);
        let caps_lock = self.state.caps_lock;
        let key = self.state.last_key_pressed?;
        if shift ^ caps_lock && key.is_char() {
            key
        }
    }
}
