pub mod x86;
pub mod traits;

use crate::drivers::traits::console::Console;
use crate::drivers::traits::timer::Timer;

use traits::intterupts_controller::IntteruptsController;
use traits::cpu_controller::CPUController;

pub trait Arch {
    // Arch traits
    type CPUController: CPUController;
    type IntteruptsController: IntteruptsController;

    // Arch specific drivers traits
    type Console: Console;
    type Timer: Timer;

    fn init() -> Self;
}
