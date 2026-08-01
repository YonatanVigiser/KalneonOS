#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(local_waker)]
#![feature(abi_x86_interrupt)]
#![feature(unsafe_cell_access)]
#![feature(trait_alias)]

pub mod drivers;
pub mod interrupt;
pub mod memory;
pub mod arch;
pub mod platform;
pub mod task;
pub mod time;
pub mod common;
pub mod dev;

extern crate alloc;

#[unsafe(link_section = ".multiboot")]
#[used]
static MULTIBOOT_HEADER: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/multiboot_header.bin"))
    .len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/multiboot_header.bin"));

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn main(boot_magic: u32, boot_info_ptr: u32) -> ! {
    interrupt::disable();
    DEVICE_REGISTRY.read().register(Arc::new(Vga::init(80, 25)));
    common::log::init_logger();
    let boot_info = arch::init_boot(boot_magic, boot_info_ptr);
    memory::init(&boot_info.mmap);
    let acpi = platform::acpi::init_platform_info(boot_info.rsdt_addr, boot_info.rsdt_revision);
    drivers::init();
    interrupt::init_global(&acpi.interrupt_model);
    let processor_info = acpi.processor_info.as_ref().expect("No processor info!");
    arch::init_cpu(processor_info.boot_processor.processor_uid, 0);
    arch::init_smp(processor_info);
    task::executor::Executor::init(arch::cores_count());
    ap_main();
}

// This is called from init_smp, and all cores should enter this when they are fully initilized
pub fn ap_main() -> ! {
    interrupt::enable();
    task::executor::EXECUTOR.wait().run()
}

pub async fn kernel_init_task() {
    task::executor::EXECUTOR.wait().spawn(Task::new(time::timer::Timer::wake_timers()));
}

use core::panic::PanicInfo;

use alloc::sync::Arc;
use x86_64::instructions::interrupts;

use crate::task::{Task, executor::EXECUTOR};

use self::dev::registry::DEVICE_REGISTRY;
use self::drivers::display::vga::Vga;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { arch::halt_smp(); }
    log::error!("{}", info);
    halt_loop()
}

pub fn halt_loop() -> ! {
    interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Heap allocation failed: {:?}", layout)
}
