pub mod x86;

use crate::drivers::traits::console::ConsoleImpl;
use crate::drivers::traits::timer::TimerImpl;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize) -> Self;

    // Arch-specific / boot dependent drivers init
    fn init_console(&self) -> ConsoleImpl;
    fn init_timer(&self) -> TimerImpl;
}
