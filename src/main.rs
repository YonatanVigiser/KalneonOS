#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(local_waker)]
#![feature(abi_x86_interrupt)]
#![feature(unsafe_cell_access)]
#![feature(trait_alias)]
#![feature(sync_unsafe_cell)]
#![feature(never_type)]

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
    memory::heap::init();
    common::log::init_logger();
    log::info!("Heap was initilized");
    arch::init_cpu(0, CpuId(0));
    drivers::init_stage1();
    let boot_info = arch::init_boot(boot_magic, boot_info_ptr);
    memory::init(&boot_info.mmap);
    let acpi = platform::acpi::init_platform_info(boot_info.rsdt_addr, boot_info.rsdt_revision);
    interrupt::init_global(&acpi.interrupt_model);
    drivers::init_stage2();
    let processor_info = acpi.processor_info.as_ref().expect("No processor info!");
    arch::init_smp(processor_info);
    task::executor::Executor::init(arch::cores_count());
    LOGGER.auto_flush.store(false, Ordering::Release);
    EXECUTOR.get().unwrap().spawn(Task::new(LOGGER.log_task()));
    drivers::init_stage3();
    ap_main();
}

// This is called from init_smp, and all cores should enter this when they are fully initilized
pub fn ap_main() -> ! {
    interrupt::enable();
    log::info!("Hello from core {}", current_cpu().logical_id);
    task::executor::EXECUTOR.wait().run()
}

pub async fn kernel_init_task() {
    task::executor::EXECUTOR.wait().spawn(Task::new(time::timer::Timer::wake_timers()));
}

use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use log::Log;
use x86_64::instructions::interrupts;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    static PANICKING: AtomicBool = AtomicBool::new(false);
    if PANICKING.swap(true, Ordering::AcqRel) { halt_loop() }
    unsafe { arch::halt_smp(); }
    if let Some(mut panic_log_sink) = drivers::panic_log_sink() {
        let _ = writeln!(panic_log_sink, "{info}");
        let _ = writeln!(panic_log_sink, "--- log flush ---");
    }
    LOGGER.flush();
    halt_loop()
}

use crate::task::Task;

use self::arch::cpu::{CpuId, current_cpu};
use self::common::log::LOGGER;
use self::task::executor::EXECUTOR;

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
