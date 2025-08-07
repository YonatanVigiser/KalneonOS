#![no_std]
#![no_main]
#![feature(fmt_internals)]

mod video;

use crate::video::vga::{ VGA, VgaColor };
use core::panic::PanicInfo;
//use core::fmt::Write;
use core::fmt::{Arguments, Write};

pub const TERMINAL_WIDTH: u8 = 80;
pub const TERMINAL_HEIGHT: u8 = 25;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
  let mut vga = VGA::new(TERMINAL_WIDTH, TERMINAL_HEIGHT);
  vga.set_colors(VgaColor::Black, VgaColor::White).clear();
  let val = 5;
  write!(&mut vga, "kernel paniced {}", val);
  loop {}
}
