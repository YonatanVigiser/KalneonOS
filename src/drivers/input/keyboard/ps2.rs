use core::pin;
use core::sync::atomic::{AtomicU32, Ordering};

use alloc::sync::Arc;
use futures_util::future::select;
use pc_keyboard::{ScancodeSet, ScancodeSet2};
use ps2::Controller;
use ps2::flags::{ControllerConfigFlags, ControllerStatusFlags, KeyboardLedFlags};
use spin::Mutex;

use crate::dev::registry::DEVICE_REGISTRY;
use crate::drivers::input::keyboard::{KEYBOARD_GLOBAL_STATE, KEYBOARD_INPUT_HUB};
use crate::interrupt::{self, GlobalInterruptController, GlobalInterruptSource, InterruptListener};
use crate::task::{Task, yield_now};
use crate::task::executor::EXECUTOR;

use super::{KeyboardDevice, LedState};

impl From<LedState> for KeyboardLedFlags {
    fn from(value: LedState) -> Self {
        let mut result = KeyboardLedFlags::empty();
        result.set(KeyboardLedFlags::CAPS_LOCK, value.contains(LedState::CAPS_LOCK));
        result.set(KeyboardLedFlags::NUM_LOCK, value.contains(LedState::NUM_LOCK));
        result.set(KeyboardLedFlags::SCROLL_LOCK, value.contains(LedState::SCROLL_LOCK));
        result
    }
}

struct I8042Ps2Driver {
    controller: Mutex<Controller>,
    has_keyboard: bool,
    has_mouse: bool,
    keyboard_connected: bool,
    _mouse_connected: bool,
    keyboard_timeout_count: AtomicU32,
    mouse_timeout_count: AtomicU32,
}

impl I8042Ps2Driver {
    fn init(keyboard_interrupt: GlobalInterruptSource, mouse_interrupt: GlobalInterruptSource) -> Option<Arc<Self>> {
        let mut controller = unsafe { Controller::new() };

        controller.disable_keyboard().ok()?;
        controller.disable_mouse().ok()?;

        let _ = controller.read_data();

        let mut config = controller.read_config().ok()?;
        config.set(
            ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
            | ControllerConfigFlags::ENABLE_TRANSLATE,
            false
        );
        controller.write_config(config).ok()?;

        controller.test_controller().ok()?;
        controller.write_config(config).ok()?;

        let has_mouse = if config.contains(ControllerConfigFlags::DISABLE_MOUSE) {
            controller.enable_mouse().ok()?;
            config = controller.read_config().ok()?;
            !config.contains(ControllerConfigFlags::DISABLE_MOUSE)
        } else {
            false
        };
        controller.disable_mouse().ok()?;

        let has_keyboard = controller.test_keyboard().is_ok();
        let has_mouse = if has_mouse { controller.test_mouse().is_ok() } else { false };

        if !has_keyboard && !has_mouse { return None; }
        
        let interrupt_guard = interrupt::guard::InterruptGuard::new();
        config = controller.read_config().ok()?;
        let keyboard_connected = if has_keyboard {
            controller.enable_keyboard().ok()?;
            config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
            config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
            let test_passed = controller.keyboard().reset_and_self_test().is_ok();
            test_passed && controller.keyboard().enable_scanning().is_ok()
        } else { false };
        let _mouse_connected = if has_mouse {
            controller.enable_mouse().ok()?;
            config.set(ControllerConfigFlags::DISABLE_MOUSE, false);
            config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, true);
            let test_passed = controller.mouse().reset_and_self_test().is_ok();
            test_passed && controller.mouse().enable_data_reporting().is_ok()
        } else { false };

        controller.write_config(config).ok()?;
        drop(interrupt_guard);

        let driver = Arc::new(Self {
            controller: Mutex::new(controller),
            has_keyboard, has_mouse,
            keyboard_connected, _mouse_connected,
            keyboard_timeout_count: AtomicU32::new(0),
            mouse_timeout_count: AtomicU32::new(0),
        });

        let global_interrupt_controller = DEVICE_REGISTRY.read().query::<dyn GlobalInterruptController>().get(0).expect("No GlobalInterruptController").clone();
        let keyboard_listener = if driver.has_keyboard && let Ok((keyboard_target, keyboard_slot)) = global_interrupt_controller.allocate_target() {
            global_interrupt_controller.route(keyboard_interrupt, keyboard_target).expect("GlobalInterruptController routeing failed!");
            global_interrupt_controller.unmask(keyboard_interrupt).expect("GlobalInterruptController unmasing error!");
            Some(keyboard_slot)
        } else { None }.map(|slot| slot.listen());
        let mouse_listener = if driver.has_mouse && let Ok((mouse_target, mouse_slot)) = global_interrupt_controller.allocate_target() {
            global_interrupt_controller.route(mouse_interrupt, mouse_target).expect("GlobalInterruptController routeing failed!");
            global_interrupt_controller.unmask(mouse_interrupt).expect("GlobalInterruptController unmasing error!");
            Some(mouse_slot)
        } else { None }.map(|slot| slot.listen());

        EXECUTOR.get().expect("No Executor!").spawn(Task::new(driver.clone().handle_task(keyboard_listener, mouse_listener)));

        Some(driver)
    }


    async fn handle_task(self: Arc<Self>, mut keyboard_listener: Option<InterruptListener>, mut mouse_listener: Option<InterruptListener>) {
        const BYTES_READ_YIELD_CAP: usize = 100;
        let mut scancode_set = ScancodeSet2::new();
        let mut keyboard_buff = [0; BYTES_READ_YIELD_CAP];
        let mut mouse_buff = [0; BYTES_READ_YIELD_CAP];
        loop {
            let mut total_read = 0;
            let mut keyboard_count = 0;
            let mut mouse_count = 0;
            {
                let mut controller = self.controller.lock();
                while total_read < BYTES_READ_YIELD_CAP && controller.read_status().contains(ControllerStatusFlags::OUTPUT_FULL) {
                    let from_keyboard = !controller.read_status().contains(ControllerStatusFlags::MOUSE_OUTPUT_FULL);
                    total_read += 1;
                    match controller.read_data() {
                        Ok(byte) if from_keyboard => { keyboard_buff[keyboard_count] = byte; keyboard_count += 1; }
                        Ok(byte) => { mouse_buff[mouse_count] = byte; mouse_count += 1 }
                        Err(_) if from_keyboard => { self.keyboard_timeout_count.fetch_add(1, Ordering::Relaxed); }
                        Err(_) => { self.mouse_timeout_count.fetch_add(1, Ordering::Relaxed); }
                    }
                }
            }

            for &byte in &keyboard_buff[..keyboard_count] {
                if let Ok(Some(event)) = scancode_set.advance_state(byte) {
                    KEYBOARD_GLOBAL_STATE.update(event.clone());
                    KEYBOARD_INPUT_HUB.push(event);
                }
            }

            if total_read == BYTES_READ_YIELD_CAP {
                log::warn!("PS/2 Device input buffer is stuck at full!");
                yield_now().await;
                continue;
            }

            match (keyboard_listener.as_mut(), mouse_listener.as_mut()) {
                (Some(x), Some(y)) => { select(pin::pin!(x.wait()), pin::pin!(y.wait())).await; }
                (Some(x), None) | (None, Some(x)) => { x.wait().await; }
                (None, None) => unreachable!("At least one should be Some"),
            };
        }
    }
}

impl KeyboardDevice for I8042Ps2Driver {
    fn connected(&self) -> bool {
        self.keyboard_connected
    }

    fn set_leds(&self, state: LedState) -> bool {
        let mut controller = self.controller.lock();
        let _ = controller.keyboard().disable_scanning().is_ok();
        let leds_written = controller.keyboard().set_leds(state.into()).is_ok();
        let scan_enable = controller.keyboard().enable_scanning().is_ok();
        if !scan_enable { log::warn!("PS/2 Keyboard scan enable failed!") }
        leds_written
    }
}

pub fn init(keyboard_interrupt: GlobalInterruptSource, mouse_interrupt: GlobalInterruptSource) {
    match I8042Ps2Driver::init(keyboard_interrupt, mouse_interrupt) {
        Some(dev) => { DEVICE_REGISTRY.write().register::<dyn KeyboardDevice>(dev); },
        None => { log::info!("PS/2 Driver init failed"); },
    }
}
