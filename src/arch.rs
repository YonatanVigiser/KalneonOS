pub mod interrupts;
pub mod gdt;

pub fn init() {
    unsafe { gdt::load(); }
}

