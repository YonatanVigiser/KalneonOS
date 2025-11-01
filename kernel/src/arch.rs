pub mod x86;

use crate::drivers::traits::console::{SerialConsole, VideoConsole};
use crate::drivers::traits::console::keyboard::KeyboardDriver;
use crate::drivers::traits::timer::Timer;

use core::panic::PanicInfo;

use alloc::boxed::Box;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    fn panic(info: &PanicInfo) -> !;

    // Arch-specific drivers access - returns static reference to arch drivers
    fn arch_drivers() -> Option<&'static mut ArchDrivers>;

    fn take_arch_drivers() -> ArchDrivers;
}

pub struct ArchDrivers {
    pub video: Option<Box<dyn VideoConsole>>,
    pub serial: Option<Box<dyn SerialConsole>>,
    pub keyboard: Option<Box<dyn KeyboardDriver>>,
    pub timer: Box<dyn Timer>,
}
