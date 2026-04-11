pub mod apic;
pub mod idt;

pub fn init() {
    idt::init();
}
