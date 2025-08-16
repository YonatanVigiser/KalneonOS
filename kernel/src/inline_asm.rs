use crate::idt::Idtr;
use core::arch::asm;

pub unsafe fn lidt(idtr: &Idtr) {
  unsafe { asm!(
    "lidt [{}]",
    in(reg) idtr,
    options(nomem, nostack),
  ); }
}

pub unsafe fn sti() {
  unsafe { asm!("sti", options(nomem, nostack)); }
}

pub unsafe fn cli() {
  unsafe { asm!("cli", options(nomem, nostack)); }
}

pub fn inb(port: u16) -> u8 {
  let inb: u8;
  unsafe { asm!(
    "in {to}, dx",
    to = out(reg_byte) inb,
    in("dx") port,
    options(nomem, nostack),
  ); }
  inb
}

pub fn outb(port: u16, value: u8) {
  unsafe { asm!(
    "out dx, {value}",
    value = in(reg_byte) value,
    in("dx") port,
    options(nomem, nostack),
  ); }
}

pub fn io_wait() {
  let mut _unused: u8 = 0;
  unsafe { asm!(
    "in {}, 0x80",
    inout(reg_byte) _unused,
    options(nomem, nostack),
  ); }
}

