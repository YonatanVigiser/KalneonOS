#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

pub mod acpi;
pub mod boot_info;
pub mod drivers;
pub mod gdt;
pub mod interrupts;
pub mod logging;
pub mod memory;
pub mod traits;

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
    interrupts::init();
    logging::init_boot_logger();
    let boot_info = boot_info::load(boot_magic, boot_info_ptr);
    memory::init(&boot_info.mmap);
    let acpi = acpi::platform_info(boot_info.rsdt_addr, boot_info.rsdt_revision);
    drivers::hpet::init_hpet(&acpi.tables).expect("HPET init failed");
    log::info!("Kernel booting...");
    loop {
        log::info!("Time: {}", drivers::uptime_nano());
    }
    halt_loop()
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
