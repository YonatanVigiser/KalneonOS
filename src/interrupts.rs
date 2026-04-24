pub mod apic;
pub mod idt;

use acpi::platform::InterruptModel;
use x2apic::lapic::LocalApic;

pub fn init_local() -> LocalApic {
    idt::init();
    let mut lapic = apic::init_lapic();
    apic::init_lapic_timer(&mut lapic, 10000);
    lapic
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn init_global(interrupt_model: &InterruptModel) {
    match interrupt_model {
        InterruptModel::Apic(apic) => apic::set_lapic_addr(apic.local_apic_address as usize),
        _ => panic!("Unsupported interrupts mode!"),
    };
}
