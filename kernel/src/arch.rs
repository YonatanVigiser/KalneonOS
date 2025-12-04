pub mod x86;

use crate::drivers::traits::console::{SerialConsole, VideoConsole};
use crate::drivers::traits::console::keyboard::KeyboardDriver;
use crate::drivers::traits::timer::Timer;

use core::panic::PanicInfo;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize);

    fn panic(info: &PanicInfo) -> !;

    fn with_keyboard<F, R>(f: F) -> Option<R> 
        where F: FnOnce(&mut dyn KeyboardDriver) -> R;

    fn with_timer<F, R>(f: F) -> Option<R> 
        where F: FnOnce(&mut dyn Timer) -> R;

    fn with_serial<F, R>(f: F) -> Option<R> 
        where F: FnOnce(&mut dyn SerialConsole) -> R;

    fn with_video<F, R>(f: F) -> Option<R> 
        where F: FnOnce(&mut dyn VideoConsole) -> R;
}
