use crate::Arch;
use crate::drivers::traits::console::ConsoleImpl;
use crate::drivers::traits::timer::TimerImpl;

pub struct DeviceManager {
    pub console: ConsoleImpl,
    pub timer: TimerImpl,
}

impl DeviceManager {
    pub fn init(arch: &impl Arch) -> Self {
        Self {
            console: arch.init_console(),
            timer: arch.init_timer(),
        }
    }
}
