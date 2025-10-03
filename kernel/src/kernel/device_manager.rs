use crate::Arch;
use crate::drivers::traits::console::{VideoConsoleImpl, SerialConsoleImpl};
use crate::drivers::traits::timer::TimerImpl;

pub struct DeviceManager {
    pub video_console: VideoConsoleImpl,
    pub serial_console: SerialConsoleImpl,
    pub timer: TimerImpl,
}

impl DeviceManager {
    pub fn init(arch: &impl Arch) -> Self {
        Self {
            video_console: arch.init_video_console(),
            serial_console: arch.init_serial_console(),
            timer: arch.init_timer(),
        }
    }
}
