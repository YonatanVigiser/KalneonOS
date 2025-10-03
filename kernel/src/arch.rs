pub mod x86;

use crate::drivers::traits::console::{VideoConsoleImpl, SerialConsoleImpl};
use crate::drivers::traits::timer::TimerImpl;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    // Arch-specific / boot dependent drivers init
    fn init_video_console(&self) -> VideoConsoleImpl;
    fn init_serial_console(&self) -> SerialConsoleImpl;
    fn init_timer(&self) -> TimerImpl;
}
