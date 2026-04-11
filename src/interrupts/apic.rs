use acpi::platform::interrupt::{Apic, IoApic};
use pic8259::ChainedPics;
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};

pub const SUPRIOUS_VECTOR: u8 = 0xFF;
pub const APIC_ERROR_VECTOR: u8 = 0xFE;
pub const TIMER_VECOTR: u8 = 0x20;

pub fn init_lapic(apic: Apic) -> Result<LocalApic, &'static str> {
    disable_pic();
    LocalApicBuilder::new()
        .set_xapic_base(apic.local_apic_address)
        .spurious_vector(SUPRIOUS_VECTOR as usize)
        .error_vector(APIC_ERROR_VECTOR as usize)
        .timer_vector(TIMER_VECOTR as usize)
        .timer_mode(TimerMode::OneShot)
        .timer_divide(TimerDivide::Div16)
        .build()
}

fn disable_pic() {
    let mut pic = unsafe { ChainedPics::new(0x20, 0x28) };
    unsafe {
        pic.disable();
    }
}
