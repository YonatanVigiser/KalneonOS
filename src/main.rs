#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(local_waker)]
#![feature(abi_x86_interrupt)]
#![feature(unsafe_cell_access)]

pub mod acpi;
pub mod boot_info;
pub mod drivers;
pub mod gdt;
pub mod interrupts;
pub mod logging;
pub mod memory;
pub mod cpu_local;
pub mod smp;
pub mod traits;
pub mod task;

extern crate alloc;

#[unsafe(link_section = ".multiboot")]
#[used]
static MULTIBOOT_HEADER: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/multiboot_header.bin"))
    .len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/multiboot_header.bin"));


#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn main(boot_magic: u32, boot_info_ptr: u32) -> ! {
    unsafe {
        gdt::load();
    }
    logging::init_boot_logger();
    let boot_info = boot_info::load(boot_magic, boot_info_ptr);
    memory::init(&boot_info.mmap);
    let acpi = acpi::platform_info(boot_info.rsdt_addr, boot_info.rsdt_revision);
    drivers::hpet::init_hpet(&acpi.tables).expect("HPET init failed");
    interrupts::init_global(&acpi.interrupt_model);
    acpi::ACPI.call_once(|| acpi);
    let lapic = interrupts::init_local();
    let proccessors_info = &acpi::ACPI.get().unwrap().processor_info.as_ref().expect("No proccessor info found in ACPI tables!");
    cpu_local::init(proccessors_info.boot_processor.processor_uid, 0, lapic);
    let lapic = &mut cpu_local::current_cpu().lapic;
    log::info!("Starting SMP...");
    unsafe { smp::start(lapic, proccessors_info); }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    halt_loop()
}

pub fn halt_loop() -> ! {
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
