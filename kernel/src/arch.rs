pub mod x86;

use self::traits::*;
use drivers::traits::*;

pub trait Arch {
    // Arch traits:
    type CPU: CPU;
    type IntteruptsController: IntteruptsController;

    // Arch specific drivers traits:
    type Console: Console;
    type Timer: Timer;

    // Constructor:
    pub init() -> Self;
}
