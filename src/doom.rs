use core::ops::DerefMut;
use futures_util::StreamExt;
use neurodoom::{Button, Buttons, ClassicEngine, PeerId, PlayerAction};
use pc_keyboard::{KeyCode, KeyState};
use spin::Mutex;

use crate::dev::registry::DEVICE_REGISTRY;
use crate::drivers::display::framebuffer::Framebuffer;
use crate::drivers::input::keyboard::{KEYBOARD_GLOBAL_STATE, KEYBOARD_INPUT_HUB};
use crate::task::{Task, yield_now};
use crate::task::executor::EXECUTOR;

static WAD: &[u8] = include_bytes!("../assets/doom/DOOM.WAD");
static LAST_KEY: Mutex<u8> = Mutex::new(0);

pub async fn run_doom() {
    //EXECUTOR.get().unwrap().spawn(Task::new(doom_buttons_update()));
    let mut guard = DEVICE_REGISTRY.read().try_acquire_all::<Framebuffer>();
    let framebuffer = guard.get_mut(0).expect("No framebuffer");
    let mut engine = ClassicEngine::new(WAD, "E1M1").expect("doom engine init failed");
    let peer = PeerId(0);
    let mut action_builder = ActionBuilder { turn_held: 0 };
    let scaler = Scaler::new(framebuffer.width(), framebuffer.height(), true);
    loop {
        engine.tick_single(peer, action_builder.build(Keys::build(), *LAST_KEY.lock()));
        scaler.draw(engine.framebuffer(), framebuffer.deref_mut());
        framebuffer.flush();
        yield_now().await;
    }
}

pub async fn doom_buttons_update() {
    let mut reader = KEYBOARD_INPUT_HUB.subscribe();
    loop {
        let key_event = reader.next().await.unwrap();
        let new = match key_event.code {
            KeyCode::Key1 => 1,
            KeyCode::Key2 => 2,
            KeyCode::Key3 => 3,
            KeyCode::Key4 => 4,
            KeyCode::Key5 => 5,
            KeyCode::Key6 => 6,
            KeyCode::Key7 => 7,
            KeyCode::Key8 => 8,
            _ => 0,
        };
        if new != 0 && let KeyState::Down = key_event.state {
            *LAST_KEY.lock() = new as u8;
        }
    }
}

const FORWARD_MOVE: [i8; 2] = [0x19, 0x32];
const SIDE_MOVE:    [i8; 2] = [0x18, 0x28];
const ANGLE_TURN:   [i16; 3] = [640, 1280, 320];
const MAX_PLMOVE: i16 = FORWARD_MOVE[1] as i16;
const SLOW_TURN_TICS: u32 = 6;

pub struct Keys {
    run: bool,
    turn_left: bool,
    turn_right: bool,
    strafe: bool,
    forward: bool,
    back: bool,
    strafe_left: bool,
    strafe_right: bool,
    buttons: Buttons,
}

impl Keys {
    fn build() -> Self {
        let run = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::LShift);
        let turn_left = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::Q);
        let turn_right = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::E);
        let strafe = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::LAlt);
        let forward = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::W);
        let back = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::S);
        let strafe_left = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::A);
        let strafe_right = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::D);
        let mut buttons = Buttons::default();
        buttons.set(Button::Attack, KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::LControl));
        buttons.set(Button::Use, KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::Spacebar));
        buttons.set(Button::Change, KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::Tab));
        Self {
            run,
            turn_left,
            turn_right,
            strafe,
            forward,
            back,
            strafe_left,
            strafe_right,
            buttons,
        }
    }
}

pub struct ActionBuilder {
    turn_held: u32,
}

impl ActionBuilder {
    pub fn build(&mut self, k: Keys, weapon_select: u8) -> PlayerAction {
        let speed = k.run as usize;

        if k.turn_left || k.turn_right {
            self.turn_held += 1;
        } else {
            self.turn_held = 0;
        }
        let tspeed = if self.turn_held < SLOW_TURN_TICS { 2 } else { speed };

        let mut forward: i16 = 0;
        let mut side: i16 = 0;
        let mut angle_turn: i16 = 0;

        if k.strafe {
            if k.turn_right { side += SIDE_MOVE[speed] as i16; }
            if k.turn_left  { side -= SIDE_MOVE[speed] as i16; }
        } else {
            if k.turn_right { angle_turn -= ANGLE_TURN[tspeed]; }
            if k.turn_left  { angle_turn += ANGLE_TURN[tspeed]; }
        }

        if k.forward      { forward += FORWARD_MOVE[speed] as i16; }
        if k.back         { forward -= FORWARD_MOVE[speed] as i16; }
        if k.strafe_right { side += SIDE_MOVE[speed] as i16; }
        if k.strafe_left  { side -= SIDE_MOVE[speed] as i16; }

        PlayerAction {
            forward_move: forward.clamp(-MAX_PLMOVE, MAX_PLMOVE) as i8,
            side_move: side.clamp(-MAX_PLMOVE, MAX_PLMOVE) as i8,
            angle_turn,
            buttons: k.buttons,
            weapon_select,
        }
    }
}

use alloc::vec::Vec;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

const SRC_W: usize = 320;
const SRC_H: usize = 200;

pub struct Scaler {
    area: Rectangle,
    x_map: Vec<u16>,
    y_map: Vec<u16>,
}

impl Scaler {
    /// `aspect` corrects Doom's non-square pixels (200 -> 240 virtual rows).
    pub fn new(fb_w: u32, fb_h: u32, aspect: bool) -> Self {
        let virt_h = if aspect { SRC_H * 6 / 5 } else { SRC_H };
        let scale = (fb_w as usize / SRC_W).min(fb_h as usize / virt_h).max(1);

        let dst_w = (SRC_W * scale).min(fb_w as usize);
        let dst_h = (virt_h * scale).min(fb_h as usize);

        let x_map = (0..dst_w).map(|x| (x / scale).min(SRC_W - 1) as u16).collect();
        let y_map = (0..dst_h)
            .map(|y| (y * SRC_H / dst_h).min(SRC_H - 1) as u16)
            .collect();

        let origin = Point::new(
            ((fb_w as usize - dst_w) / 2) as i32,
            ((fb_h as usize - dst_h) / 2) as i32,
        );

        Self {
            area: Rectangle::new(origin, Size::new(dst_w as u32, dst_h as u32)),
            x_map,
            y_map,
        }
    }

    pub fn draw<D>(&self, src: &[u8], target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb888>,
    {
        debug_assert_eq!(src.len(), SRC_W * SRC_H * 4);

        let pixels = self.y_map.iter().flat_map(move |&sy| {
            let row = &src[sy as usize * SRC_W * 4..][..SRC_W * 4];
            self.x_map.iter().map(move |&sx| {
                let i = sx as usize * 4;
                Rgb888::new(row[i], row[i + 1], row[i + 2])
            })
        });

        target.fill_contiguous(&self.area, pixels)
    }
}
