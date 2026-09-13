use bitflags::bitflags;
use spin::Mutex;
use pc_keyboard::{KeyCode, KeyEvent, KeyState};

use crate::dev::registry::DEVICE_REGISTRY;
use crate::interrupt::apic::isa_irq_to_gsi;

use super::{InputHub, Reader};

pub mod ps2;

const PS2_KEYBOARD_ISA: u8 = 0x1;
const PS2_MOUSE_ISA: u8 = 0xC;

pub(super) fn init() {
    ps2::init(isa_irq_to_gsi(PS2_KEYBOARD_ISA), isa_irq_to_gsi(PS2_MOUSE_ISA));
}

pub static KEYBOARD_INPUT_HUB: InputHub<KeyEvent> = InputHub::new();
pub static KEYBOARD_GLOBAL_STATE: KeyboardState = KeyboardState::new();

const KEYCODES_NUM: usize = KeyCode::Unknown as usize + 1;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct LedState: u8 {
        const CAPS_LOCK   = 0b001;
        const NUM_LOCK    = 0b010;
        const SCROLL_LOCK = 0b100;
    }
}

pub trait KeyboardDevice: Send + Sync {
    fn connected(&self) -> bool;
    fn set_leds(&self, state: LedState) -> bool;
}

pub struct KeyboardState {
    states: Mutex<[bool; KEYCODES_NUM]>,
    state_update_input_hub: InputHub<KeyEvent>,
    leds_state: Mutex<LedState>,
}

impl KeyboardState {
    const fn new() -> Self {
        Self { states: Mutex::new([false; KEYCODES_NUM]), state_update_input_hub: InputHub::new(), leds_state: Mutex::new(LedState::empty()) }
    }

    fn update(&self, event: KeyEvent) {
        let index = event.code as usize;
        let pressed = {
            let mut states = self.states.lock();
            let prev = states[index];
            let new = match event.state {
                KeyState::Up => false,
                KeyState::Down => true,
                KeyState::SingleShot => prev,
            };
            states[index] = new;
            if prev == new { return; }
            new
        };
        if pressed && let Ok(bit) = event.code.try_into() {
            let leds = {
                let mut leds = self.leds_state.lock();
                leds.toggle(bit);
                *leds
            };
            let keyboards = DEVICE_REGISTRY.read().query::<dyn KeyboardDevice>();
            for keyboard in keyboards {
                keyboard.set_leds(leds);
            }
        }
        self.state_update_input_hub.push(event);
    }

    pub fn subscribe_for_state_updates(&self) -> Reader<KeyEvent> {
        self.state_update_input_hub.subscribe()
    }

    pub fn is_pressed(&self, code: KeyCode) -> bool {
        self.states.lock()[code as usize]
    }
}

impl TryFrom<KeyCode> for LedState {
    type Error = ();
    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::CapsLock => Ok(LedState::CAPS_LOCK),
            KeyCode::NumpadLock => Ok(LedState::NUM_LOCK),
            KeyCode::ScrollLock => Ok(LedState::SCROLL_LOCK),
            _ => Err(()),
        }
    }
}
