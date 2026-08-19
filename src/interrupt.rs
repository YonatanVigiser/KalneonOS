pub mod handlers;
pub mod guard;
pub mod mutex;
mod apic;

pub const SUPRIOUS_VECTOR: u8 = 0xFF;
pub const CONTROLLER_ERROR_VECTOR: u8 = 0xFE;
pub const TIMER_VECOTR: u8 = 0x30;

pub fn init_global(interrupts_model: &InterruptModel) {
    match interrupts_model {
        InterruptModel::Apic(addr) => apic::set_lapic_addr(addr.local_apic_address as usize),
        _ => panic!("Unsupported interrupts model"),
    };
}

pub fn init_local() -> LocalApic {
    let mut lapic = apic::init_lapic();
    apic::init_lapic_timer(&mut lapic, 1000000);
    lapic
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

pub fn set(enabled: bool) {
    if enabled { enable() }
}

pub fn are_enabled() -> bool {
    x86_64::instructions::interrupts::are_enabled()
}

use acpi::platform::InterruptModel;
use x2apic::lapic::LocalApic;
