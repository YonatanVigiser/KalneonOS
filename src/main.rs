#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

pub mod heap;
pub mod types;
pub mod drivers;
pub mod memory;
pub mod arch;
pub mod boot_info;
pub mod logging;

extern crate alloc;

#[unsafe(link_section = ".multiboot")]
#[used]
static MULTIBOOT_HEADER: [u8; 72] = *include_bytes!(concat!(env!("OUT_DIR"), "/multiboot_header.bin"));

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn main() -> ! {
    arch::init();
    heap::init();
    drivers::init();
    logging::init_boot_logger();
    log::info!("Kernel booting...");
    loop { }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    loop { }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
