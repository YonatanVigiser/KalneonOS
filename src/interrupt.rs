pub mod handlers;

use acpi::platform::InterruptModel;
use x2apic::lapic::LocalApic;

pub fn init_local() -> LocalApic {
    crate::platform::idt::init();
    let mut lapic = crate::platform::apic::init_lapic();
    crate::platform::apic::init_lapic_timer(&mut lapic, 10000);
    lapic
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

pub fn init_global(interrupt_model: &InterruptModel) {
    match interrupt_model {
        InterruptModel::Apic(apic) => {
            crate::platform::apic::set_lapic_addr(apic.local_apic_address as usize)
        }
        _ => panic!("Unsupported interrupts mode!"),
    };
}

use crate::platform::cpu::current_cpu;
use core::ops::{Deref, DerefMut};

pub struct InterruptSafe<T>(T);

impl<T> InterruptSafe<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn get(&mut self) -> InterruptGuard<'_, T> {
        InterruptGuard(&mut self.0)
    }
}

pub struct InterruptGuard<'a, T>(&'a mut T);

impl<'a, T> Deref for InterruptGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let cpu = current_cpu();
        if cpu.interrupt_depth() == 0 { disable(); }
        cpu.enter_interrupt();
        self.0
    }
}

impl<'a, T> DerefMut for InterruptGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let cpu = current_cpu();
        if cpu.interrupt_depth() == 0 { disable(); }
        cpu.enter_interrupt();
        self.0
    }
}

impl<'a, T> Drop for InterruptGuard<'a, T> {
    fn drop(&mut self) {
        let cpu = current_cpu();
        cpu.leave_interrupt();
        if cpu.interrupt_depth() == 0 { enable(); }
    }
}
