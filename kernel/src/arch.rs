pub mod x86;

use crate::drivers::traits::console::{VideoConsole, SerialConsole};
use crate::drivers::traits::timer::Timer;

use core::panic::PanicInfo;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    fn panic(&mut self, info: &PanicInfo) -> !;

    // Arch-specific drivers access
    fn video<'a>(&'a mut self) -> Option<&'a mut dyn VideoConsole>;
    fn serial<'a>(&'a mut self) -> Option<&'a mut dyn SerialConsole>;
    fn timer<'a>(&'a mut self) -> &'a mut dyn Timer;
}

