#![no_std]
#![no_main]

mod arch;
mod boot;
mod drivers;
mod kernel;
mod utils;

#[unsafe(no_mangle)]
#[cfg(target_arch = "x86")]
pub extern "C" fn _start(boot_info_ptr: usize) -> ! {
    let arch = arch::x86::ArchX86::init();
    kernel::kmain(arch, boot_info_ptr)
}
