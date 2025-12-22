use super::idt::Idtr;
use core::arch::asm;

#[inline]
pub unsafe fn lidt(idtr: &Idtr) {
    unsafe {
        asm!(
          "lidt [{}]",
          in(reg) idtr,
          options(nomem, nostack),
        );
    }
}

#[inline]
pub unsafe fn sti() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

#[inline]
pub unsafe fn cli() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}


#[inline]
pub fn flags() -> usize {
    let flags;
    unsafe {
        asm!(
            "pushfd",
            "pop {to}",
            to = out(reg) flags,
            options(nomem),
        );
    }
    flags
}

#[inline]
pub fn interrupts_enabled() -> bool {
    (flags() & 0x200) != 0
}

#[inline]
pub fn inb(port: u16) -> u8 {
    let inb: u8;
    unsafe {
        asm!(
          "in {to}, dx",
          to = out(reg_byte) inb,
          in("dx") port,
          options(nomem, nostack),
        );
    }
    inb
}

#[inline]
pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
          "out dx, {value}",
          value = in(reg_byte) value,
          in("dx") port,
          options(nomem, nostack),
        );
    }
}

#[inline]
pub fn io_wait() {
    let mut _unused: u8 = 0;
    unsafe {
        asm!(
          "in {}, 0x80",
          inout(reg_byte) _unused,
          options(nomem, nostack),
        );
    }
}

#[inline]
pub unsafe fn halt() {
    unsafe {
        asm!("hlt", options(nomem, nostack),);
    }
}

#[inline]
pub unsafe fn enable_paging(pd_address: u32) {
    unsafe {
        asm!(
          "mov eax, {address}",
          "mov cr3, eax",
          "mov eax, cr0",
          "or eax, 0x80000001",
          "mov cr0, eax",
          address = in(reg) pd_address,
          options(nomem, nostack),
        );
    }
}
