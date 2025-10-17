use crate::Arch;
use crate::drivers::traits::console::{VideoConsoleImpl, SerialConsoleImpl};
use crate::drivers::traits::timer::TimerImpl;

pub struct DeviceManager {
    pub video_console: Option<VideoConsoleImpl>,
    pub serial_console: Option<SerialConsoleImpl>,
    pub timer: Option<TimerImpl>,
}

impl DeviceManager {
    pub fn init(arch: &impl Arch) -> Self {
        Self {
            video_console: arch.take_video_console(),
            serial_console: arch.take_serial_console(),
            timer: arch.take_timer(),
        }
    }
}
