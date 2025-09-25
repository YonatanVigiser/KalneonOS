mod device_manager;

use device_manager::DeviceManager;
use crate::arch::Arch;
use crate::arch::traits::intterupts_controller::IntteruptsController;

pub struct Kernel<A: Arch> {
    arch: A,
    device_manager: DeviceManager<A>,
}

impl<A: Arch> Kernel<A> {
    pub fn init(arch: A) -> Self {
        Self {
            arch,
            device_manager: DeviceManager::<A>::init(),
        }
    }

    pub fn run(&mut self) -> ! {
        A::IntteruptsController::enable();
        loop {}
    }

    pub fn panic(&mut self, _info: &PanicInfo) -> ! {
        loop {}
    }
}

use core::ptr::NonNull;
use crate::TargetArch;

static mut KERNEL: Option<NonNull<Kernel<TargetArch>>> = None;

pub fn kmain(arch: TargetArch) -> ! {
    let mut kernel = Kernel::init(arch);
    unsafe { KERNEL = Some(NonNull::from(&kernel)); }
    kernel.run()
}

use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    unsafe {
        if let Some(mut kernel) = KERNEL {
            kernel.as_mut().panic(info)
        }
        else {
            loop { }
        }
    }
}
