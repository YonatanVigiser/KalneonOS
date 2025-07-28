#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
  let test = 18;
  if test > 50 {
    let test = 67;
  }
  loop {}
}
