#![no_std]
#![no_main]

mod video;

use crate::video::vga::{ VGA, VgaColor, VgaCell };
use core::panic::PanicInfo;
use core::fmt::Write;

pub const TERMINAL_WIDTH: u8 = 80;
pub const TERMINAL_HEIGHT: u8 = 25;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
  let mut display = VGA::new(TERMINAL_WIDTH, TERMINAL_HEIGHT);
  display.clear_with_color(VgaColor::Green, VgaColor::Black);
  display.write_string("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27");
  loop {}
}
