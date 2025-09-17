use crate::arch::Arch;

pub struct DeviceManager<A: Arch> {
    pub console: A::Console,
    pub timer: A::Timer,
}

impl<A> DeviceManager<A: Arch> {
    pub fn init() -> Self {
        Self {
            console: A::Console::init(),
            timer: A::Timer::init(),
        }
    }
}
