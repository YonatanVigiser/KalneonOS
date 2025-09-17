mod device_manager;

use device_manager::DeviceManager;

pub struct Kernel<A: Arch> {
    arch: A,
    device_manager: DeviceManager<A>,
}

impl Kernel<A: Arch> {
    pub init(arch: Arch, boot_info_ptr: usize) -> Self {
        Self {
            arch,
            device_manager: DeviceManager<A>::init(),
        }
    }

    pub run(&mut self) -> ! {
        A::IntteruptsController::enable();
    }
}

pub fn kmain<A: Arch>(arch: A, boot_info_ptr: usize) -> ! {
    Kernel::init(arch).run()
}
