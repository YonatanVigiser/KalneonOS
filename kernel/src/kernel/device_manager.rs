use crate::arch::Arch;
use crate::drivers::traits::timer::Timer;
use crate::drivers::traits::console::Console;

pub struct DeviceManager<A: Arch> {
    pub console: A::Console,
    pub timer: A::Timer,
}

impl<A: Arch> DeviceManager<A> {
    pub fn init() -> Self {
        Self {
            console: A::Console::init(),
            timer: A::Timer::init(),
        }
    }
}
