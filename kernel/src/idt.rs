use core::arch::asm;
use core::mem::size_of;
use crate::video::vga::VGA;

const USED_INTS_NUM: usize = 48;

#[repr(C, packed)]
struct Idtr {
  idt_size: u16,
  idt_ptr: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
  isr_low: u16,
  kernel_cs: u16,
  reserved: u8,
  attributes: u8,
  isr_high: u16,
}

impl IdtEntry {
  const fn empty() -> Self {
    IdtEntry {
      isr_low: 0,
      kernel_cs: 8,
      reserved: 0,
      attributes: 0x8E,
      isr_high: 0,
    }
  }
}

#[repr(align(16))]
struct Idt ([IdtEntry; USED_INTS_NUM]);

unsafe extern "C" {
    static isr_stub_table: [u32; USED_INTS_NUM];
}

static mut IDT: Idt = Idt([IdtEntry::empty(); USED_INTS_NUM]);

pub fn init() {
  let idt_ptr: u32;
  unsafe { 
    IDT = create_idt();
    idt_ptr = &raw const IDT as *const _ as u32;
  }
  let idtr = Idtr {
    idt_size: ((USED_INTS_NUM * size_of::<IdtEntry>()) - 1) as u16,
    idt_ptr,
  };
  lidt(&idtr);
}

fn create_idt() -> Idt {
  let mut idt: Idt = Idt([IdtEntry::empty(); USED_INTS_NUM]);
  for (index, entry) in idt.0.iter_mut().enumerate() {
    let isr_addr: u32 = unsafe { isr_stub_table[index] };
    entry.isr_low = (isr_addr & 0xFFFF) as u16;
    entry.isr_high = (isr_addr >> 16) as u16;
  }
  idt
}

fn lidt(idtr: &Idtr) {
  unsafe { asm!(
    "lidt [{}]",
    in(reg) idtr,
    options(nostack, readonly),
  ); }
}

pub fn sti() {
  unsafe { asm!("sti", options(nostack, readonly)); }
}

pub fn cli() {
  unsafe { asm!("cli", options(nostack, readonly)); }
}

use crate::video::vga::VgaColor;
use core::fmt::Write;

#[unsafe(no_mangle)]
pub extern "C" fn cpu_exception_handler(error_code: u32, int_num: u32) {
  let vga = unsafe { crate::VGA_GLOBAL.get_mut() };
  vga.set_colors(VgaColor::Red, VgaColor::Black).clear();
  write!(vga, "An exception occored. Int num: {int_num}, error_code: {error_code}");
}
