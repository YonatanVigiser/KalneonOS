pub mod x86;

use crate::drivers::traits::console::{SerialConsole, VideoConsole};
use crate::drivers::traits::timer::Timer;

use core::panic::PanicInfo;
use spin::Mutex;

pub trait Arch {
    type VideoDriver: VideoConsole + 'static;
    type SerialDriver: SerialConsole + 'static;
    type TimerDriver: Timer + 'static;

    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    fn panic(&mut self, info: &PanicInfo) -> !;

    // Arch-specific drivers access - returns references to static mutexes
    fn video() -> &'static Mutex<Option<Self::VideoDriver>>;
    fn serial() -> &'static Mutex<Option<Self::SerialDriver>>;
    fn timer() -> &'static Mutex<Option<Self::TimerDriver>>;
}
