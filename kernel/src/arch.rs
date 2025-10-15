pub mod x86;

use crate::drivers::traits::console::{VideoConsole, SerialConsole};
use crate::drivers::traits::timer::Timer;

use core::panic::PanicInfo;
use core::ptr::NonNull;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    fn panic(&mut self, info: &PanicInfo) -> !;

    // Arch-specific drivers access
    fn video() -> Option<NonNull<dyn VideoConsole>>;
    fn serial() -> Option<NonNull<dyn SerialConsole>>;
    fn timer() -> NonNull<dyn Timer>;
}

