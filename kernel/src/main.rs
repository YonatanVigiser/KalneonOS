#![no_std]
#![no_main]

mod kernel;
mod arch;
mod utils;
mod boot;
mod drivers;

use kernel::kmain::kernel_main;

#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info_ptr: u32) -> ! {
  kernel_main(boot_info_ptr)
}
