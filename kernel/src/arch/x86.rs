pub mod cpu;
pub mod drivers;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod heap;

use super::{Arch, ArchDrivers};

use drivers::pit::PitTimer;
use drivers::serial::SerialDriver;
use drivers::vga::Vga;

use core::panic::PanicInfo;

use alloc::boxed::Box;

// Early boot drivers references for IRQ handlers
// This is initialized once during boot and then accessed by both IRQ handlers and kernel
pub static mut ARCH_DRIVERS: Option<ArchDrivers> = None;

pub struct ArchX86();

impl Arch for ArchX86 {
    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) -> Self {
        // Init CPU
        idt::init();

        // Init heap
        heap::init_heap();

        // Init intterupts:
        pic::init();

        // Init early drivers
        unsafe {
            ARCH_DRIVERS = Some(ArchDrivers {
                video: Some(Box::new(Vga::init(80,25))),
                serial: {
                    if let Some(driver) = SerialDriver::init() {
                        Some(Box::new(driver))
                    } else {
                        None
                    }
                },
                timer: Box::new(PitTimer::init()),
            });
        }
        if let Some(arch_drivers) = Self::arch_drivers() && let Some(video) = arch_drivers.video.as_mut() {
            let _ = video.clear().write_str("Arch init is complete!");
        }

        // Finish init - enable interrupts
        unsafe {
            cpu::sti();
        }

        Self()
    }

    fn panic(info: &PanicInfo) -> ! {
        use crate::kernel::display::color::Color;
        
        unsafe {
            cpu::cli();
        }

        if let Some(arch_drivers) = Self::arch_drivers() && let Some(video) = arch_drivers.video.as_mut() {
            video.set_bg(Color::red()).set_fg(Color::black()).clear();
            let _ = writeln!(video, "{}", info);
        }

        loop {
            core::hint::spin_loop();
        }
    }

    fn arch_drivers() -> Option<&'static mut ArchDrivers> {
        unsafe {
            let ptr = core::ptr::addr_of_mut!(ARCH_DRIVERS);
            (*ptr).as_mut()
        }
    }

    fn take_arch_drivers() -> ArchDrivers {
        unsafe {
            let ptr = core::ptr::addr_of_mut!(ARCH_DRIVERS);
            (*ptr).take().expect("Arch drivers aren't initilized!")
        }
    }
}
