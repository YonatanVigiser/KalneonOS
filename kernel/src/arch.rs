pub mod x86;

use crate::drivers::traits::console::{SerialConsole, VideoConsole};
use crate::drivers::traits::timer::Timer;

use core::panic::PanicInfo;

use alloc::boxed::Box;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    fn panic(&mut self, info: &PanicInfo) -> !;

    // Arch-specific drivers access - returns references to static mutexes
    fn video() -> Option<Box<dyn VideoConsole>>;
    fn serial() -> Option<Box<dyn SerialConsole>>;
    fn timer() -> Box<dyn Timer>;
}
