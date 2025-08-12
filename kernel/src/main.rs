#![no_std]
#![no_main]

mod video;
mod idt;
mod inline_asm;
mod types;
mod pic;

use crate::video::vga::{ VGA, VgaColor };
use crate::types::SyncUnsafeCell;
use core::fmt::Write;
use core::cell::UnsafeCell;
use core::panic::PanicInfo;

pub const TERMINAL_WIDTH: u8 = 80;
pub const TERMINAL_HEIGHT: u8 = 25;

pub static VGA_GLOBAL: SyncUnsafeCell<VGA> = SyncUnsafeCell(UnsafeCell::new(VGA::default()));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  let vga = VGA_GLOBAL.get_mut();
  let _ = vga.set_colors(VgaColor::Red, VgaColor::Black).clear();
  let _ = write!(vga, "Kernel panicked: {info}");
  loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
  *VGA_GLOBAL.get_mut() = VGA::new(80, 25);
  idt::init();
  pic::init();
  pic::unmask_irq(1);
  let vga = VGA_GLOBAL.get_mut();
  vga.set_colors(VgaColor::Black, VgaColor::White).clear().write_string("hello!").expect("An error occored in the VGA");
  unsafe { inline_asm::sti(); }
  loop {}
}
