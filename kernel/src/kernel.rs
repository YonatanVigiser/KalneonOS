mod device_manager;

use device_manager::DeviceManager;
use crate::arch::Arch;

pub struct Kernel<A: Arch> {
    arch: A,
    device_manager: DeviceManager<A>,
}

impl<A> Kernel<A: Arch> {
    pub fn init(arch: A, boot_info_ptr: usize) -> Self {
        Self {
            arch,
            device_manager: DeviceManager::<A>::init(),
        }
    }

    pub fn run(&mut self) -> ! {
        A::InterupptsController::enable();
        loop {}
    }
}

pub fn kmain<A: Arch>(arch: A, boot_info_ptr: usize) -> ! {
    Kernel::init(arch, boot_info_ptr).run()
}
