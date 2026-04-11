pub mod apic;
pub mod idt;

use acpi::platform::InterruptModel;

pub fn init(interrupt_model: &InterruptModel) {
    idt::init();
    match interrupt_model {
        InterruptModel::Apic(apic) => apic::set_lapic_addr(apic.local_apic_address as usize),
        _ => panic!("Unsupported interrupts mode!"),
    };
    let mut lapic = apic::init_lapic();
    apic::init_lapic_timer(&mut lapic, 10000);
    x86_64::instructions::interrupts::enable();
}
