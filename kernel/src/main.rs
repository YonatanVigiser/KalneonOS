#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

mod arch;
mod boot;
mod drivers;
mod kernel;
mod utils;

extern crate alloc;

use arch::Arch;

#[cfg(target_arch = "x86")]
pub type TargetArch = arch::x86::ArchX86;

#[unsafe(no_mangle)]
pub extern "C" fn main(boot_magic_val: usize, boot_info_ptr: usize) -> ! {
    let arch = TargetArch::init(boot_magic_val, boot_info_ptr);
    kernel::kmain(arch)
}
