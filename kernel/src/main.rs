#![no_std]
#![no_main]

mod video;
mod types;

use crate::video::vga::{ VGA, VgaColor, VgaCell };
use crate::types::{ SimpleMutex, SimpleOnce };
use core::panic::PanicInfo;
use core::fmt::Write;

pub const TERMINAL_WIDTH: u8 = 80;
pub const TERMINAL_HEIGHT: u8 = 25;

static VGA: SimpleOnce<SimpleMutex<VGA>> = SimpleOnce::<SimpleMutex<VGA>>::new();

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
  init_vga();
  let test = 5;
  let mut vga = get_vga();
  //write!(&mut *vga, "hello, {}", test).unwrap();
  loop {}
}

pub fn get_vga() -> &'static mut VGA {
  VGA.get().expect("VGA isn't initialized").lock()
}

fn init_vga() {
    VGA.call_once(|| {
        SimpleMutex::new(VGA::new(TERMINAL_WIDTH, TERMINAL_HEIGHT))
    });
}
