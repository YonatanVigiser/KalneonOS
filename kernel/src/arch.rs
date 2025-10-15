pub mod x86;

use crate::drivers::traits::console::{VideoConsole, SerialConsole};
use crate::drivers::traits::timer::Timer;

use core::panic::PanicInfo;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    fn panic(&mut self, info: &PanicInfo) -> !;

    // Arch-specific drivers access
    fn video() -> Option<&'static mut dyn VideoConsole>;
    fn serial() -> Option<&'static mut dyn SerialConsole>;
    fn timer() -> &'static mut dyn Timer;
}

