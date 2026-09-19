use core::ops::DerefMut;
use fugit::HertzU64;
use neurodoom::{Button, Buttons, ClassicEngine, PeerId, PlayerAction};
use pc_keyboard::{KeyCode, KeyEvent, KeyState};

use crate::dev::lease::Lease;
use crate::dev::registry::DEVICE_REGISTRY;
use crate::drivers::display::framebuffer::Framebuffer;
use crate::drivers::input::Reader;
use crate::drivers::input::keyboard::KEYBOARD_GLOBAL_STATE;
use crate::task::executor::EXECUTOR;
use crate::task::{Task, yield_now};
use crate::time::{KernelDuration, KernelInstant, uptime};

static WAD: &[u8] = include_bytes!("../assets/doom/DOOM.WAD");

pub struct DoomApp {
    framebuffer: Lease<Framebuffer>,
    engine: ClassicEngine,
    peer: PeerId,
    scaler: Scaler,
    action_builder: ActionBuilder,
    keyevent_reader: Reader<KeyEvent>,
    last_doom_tick: KernelInstant,
    last_key: u8,
}

impl DoomApp {
    pub fn new() -> Self {
        let mut guard = DEVICE_REGISTRY.read().try_acquire_all::<Framebuffer>();
        let framebuffer = guard.swap_remove(0);
        let engine = ClassicEngine::new(WAD, "E1M1").expect("doom engine init failed");
        let peer = PeerId(0);
        let action_builder = ActionBuilder { turn_held: 0 };
        let scaler = Scaler::new(framebuffer.width(), framebuffer.height(), true);
        let keyevent_reader = KEYBOARD_GLOBAL_STATE.subscribe_for_state_updates();
        Self {
            framebuffer,
            engine,
            peer,
            action_builder,
            scaler,
            keyevent_reader,
            last_doom_tick: uptime(),
            last_key: 0,
        }
    }

    pub async fn run_doom(mut self) {
        const DOOM_ENGINE_TICK: KernelDuration = HertzU64::Hz(25).to_duration();
        self.framebuffer.clear(Rgb888::BLACK);
        loop {
            if self.keyevent_reader.0.take_dropped() > 0 {
                log::warn!("keyboard event dropped");
            }
            while let Some(event) = self.keyevent_reader.0.pop() {
                let new = match event.code {
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
                if new != 0
                    && let KeyState::Down = event.state
                {
                    self.last_key = new as u8;
                }
                if let KeyCode::R = event.code {
                    self.engine.load_map(&WAD, "E1M1").unwrap();
                    self.engine.spawn_map_things(PeerId(0));
                }
            }
            let now = uptime();
            if now.checked_duration_since(self.last_doom_tick).unwrap() >= DOOM_ENGINE_TICK {
                self.last_doom_tick = now;
                let keys = Keys::build();
                let action = self.action_builder.build(keys, self.last_key);
                self.engine.tick_single(self.peer, action);
            }
            self.scaler
                .draw(self.engine.framebuffer(), self.framebuffer.deref_mut());
            self.framebuffer.flush();
            yield_now().await;
        }
    }
}

const FORWARD_MOVE: [i8; 2] = [0x19, 0x32];
const SIDE_MOVE: [i8; 2] = [0x18, 0x28];
const ANGLE_TURN: [i16; 3] = [640, 1280, 320];
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
        let turn_left = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::ArrowLeft);
        let turn_right = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::ArrowRight);
        let strafe = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::O);
        let forward = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::W);
        let back = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::S);
        let strafe_left = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::A);
        let strafe_right = KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::D);
        let mut buttons = Buttons::default();
        buttons.set(
            Button::Attack,
            KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::Spacebar),
        );
        buttons.set(
            Button::Use,
            KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::LControl),
        );
        buttons.set(
            Button::Change,
            KEYBOARD_GLOBAL_STATE.is_pressed(KeyCode::Tab),
        );
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
        let tspeed = if self.turn_held < SLOW_TURN_TICS {
            2
        } else {
            speed
        };

        let mut forward: i16 = 0;
        let mut side: i16 = 0;
        let mut angle_turn: i16 = 0;

        if k.strafe {
            if k.turn_right {
                side += SIDE_MOVE[speed] as i16;
            }
            if k.turn_left {
                side -= SIDE_MOVE[speed] as i16;
            }
        } else {
            if k.turn_right {
                angle_turn -= ANGLE_TURN[tspeed];
            }
            if k.turn_left {
                angle_turn += ANGLE_TURN[tspeed];
            }
        }

        if k.forward {
            forward += FORWARD_MOVE[speed] as i16;
        }
        if k.back {
            forward -= FORWARD_MOVE[speed] as i16;
        }
        if k.strafe_right {
            side += SIDE_MOVE[speed] as i16;
        }
        if k.strafe_left {
            side -= SIDE_MOVE[speed] as i16;
        }

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
    pub fn new(fb_w: u32, fb_h: u32, aspect: bool) -> Self {
        let (fb_w, fb_h) = (fb_w as usize, fb_h as usize);
        let virt_h = if aspect { SRC_H * 6 / 5 } else { SRC_H };

        let (dst_w, dst_h) = if fb_w * virt_h <= fb_h * SRC_W {
            (fb_w, fb_w * virt_h / SRC_W)
        } else {
            (fb_h * SRC_W / virt_h, fb_h)
        };

        let x_map = (0..dst_w).map(|x| (x * SRC_W / dst_w) as u16).collect();
        let y_map = (0..dst_h).map(|y| (y * SRC_H / dst_h) as u16).collect();

        let origin = Point::new(((fb_w - dst_w) / 2) as i32, ((fb_h - dst_h) / 2) as i32);

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

pub fn init_app() {
    let doom_app = DoomApp::new();
    EXECUTOR
        .get()
        .unwrap()
        .spawn(Task::new(doom_app.run_doom()));
}
