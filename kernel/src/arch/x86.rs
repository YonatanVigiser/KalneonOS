pub mod cpu;
pub mod drivers;
pub mod heap;
pub mod idt;
pub mod interrupts;
pub mod pic;

use drivers::pit::PitTimer;
use drivers::serial::SerialDriver;
use drivers::vga::Vga;
use drivers::ps2_keyboard::PS2Keyboard;

use core::panic::PanicInfo;

use crate::drivers::traits::console::*;
use crate::drivers::traits::console::keyboard::KeyboardDriver;

use crate::kernel::memory::frame::{MemoryType, FRAME_SIZE};
use crate::kernel::memory::map::{MemoryMap, MemoryRegion};

use spin::Mutex;
use alloc::boxed::Box;
use alloc::vec;

// Drivers:
pub static KEYBOARD: Mutex<Option<Box<dyn KeyboardDriver>>> = Mutex::new(None);
pub static VIDEO: Mutex<Option<Box<dyn VideoConsole>>> = Mutex::new(None);
pub static SERIAL: Mutex<Option<Box<dyn SerialConsole>>> = Mutex::new(None);

// Arch specific boot info:
pub static MEMORY_MAP: Mutex<Option<MemoryMap>> = Mutex::new(None);

pub struct ArchX86();

impl ArchX86 {
    fn init_drivers() {
        *VIDEO.lock() = Some(Box::from(Vga::init(80, 25)));
        if let Some(serial) = SerialDriver::init() {
            *SERIAL.lock() = Some(Box::from(serial));
            pic::unmask_irq(4); // Serial port 1
        }
        PitTimer::init();
        pic::unmask_irq(0); // Timer

        // Init ps/2 drivers:
        // Init the ps/2 controller
        if let Ok(types) = drivers::ps2::init() {
            let (keyboard_type, _mouse_type) = types;

            if let Some(keyboard_type) = keyboard_type {
                if let Ok(driver) = PS2Keyboard::init(keyboard_type) {
                    *KEYBOARD.lock() = Some(Box::from(driver));
                    pic::unmask_irq(1); // Keyboard
                }
            }
        }
    }
}

unsafe extern "C" {
    unsafe fn context_switch(old_stack_ptr: &mut usize, new_stack_ptr: usize);
    fn fake_thread_entry_stack(stack_ptr: &mut usize, entry_point: usize);
}

use super::Arch;

impl Arch for ArchX86 {
    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) {
        idt::init();

        heap::init_heap();

        pic::init();

        Self::init_drivers();

        *MEMORY_MAP.lock() = Some(vec![MemoryRegion { start: 0, frames_size: 1024 * 1024 * 20 / FRAME_SIZE, memory_type: MemoryType::KernelAddressSpace }, MemoryRegion { start: 1024 *  1024 * 20, frames_size: (usize::MAX - 1024 * 1024 * 20) / FRAME_SIZE, memory_type: MemoryType::Usable }]);

        Self::with_video(|video| {
            let _ = video.clear().write_str("Arch init is complete!");
        });

        unsafe {
            cpu::sti();
        }
    }

    fn panic(info: &PanicInfo) -> ! {
        use crate::kernel::display::color::Color;

        unsafe {
            cpu::cli();
        }

        Self::with_video(|video| {
            video.set_bg(Color::red()).set_fg(Color::black()).clear();
            let _ = writeln!(video, "{}", info);
        });

        loop {
            core::hint::spin_loop();
        }
    }

    fn take_memory_map() -> Option<MemoryMap> {
        MEMORY_MAP.lock().take()
    }

    unsafe fn context_switch(old_stack_ptr: &mut usize, new_stack_ptr: usize) {
        unsafe { context_switch(old_stack_ptr, new_stack_ptr); }
    }

    fn fake_thread_entry_stack(stack_ptr: &mut usize, entry: fn() -> !) {
        unsafe { fake_thread_entry_stack(stack_ptr, entry as usize); }
    }

    fn with_keyboard<F, R>(f: F) -> Option<R>
        where F: FnOnce(&mut dyn KeyboardDriver) -> R {
            pic::mask_irq(1);

            let result = {
                let mut guard = KEYBOARD.lock();
                guard.as_mut().map(|keyboard| f(keyboard.as_mut()))
            };

            pic::unmask_irq(1);

            result
    }

    fn with_serial<F, R>(f: F) -> Option<R>
        where F: FnOnce(&mut dyn SerialConsole) -> R {
            pic::mask_irq(3);

            let result = {
                let mut guard = SERIAL.lock();
                guard.as_mut().map(|serial| f(serial.as_mut()))
            };

            pic::unmask_irq(3);

            result
    }

    fn with_video<F, R>(f: F) -> Option<R>
        where F: FnOnce(&mut dyn VideoConsole) -> R {
            let result = {
                let mut guard = VIDEO.lock();
                guard.as_mut().map(|video| f(video.as_mut()))
            };

            result
    }
}
