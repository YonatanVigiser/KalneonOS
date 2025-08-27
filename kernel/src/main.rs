#![no_std]
#![no_main]

mod arch;
mod boot;
mod drivers;
mod kernel;
mod utils;

use kernel::kmain::kernel_main;

#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info_ptr: u32) -> ! {
    kernel_main(boot_info_ptr)
}
