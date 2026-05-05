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

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

pub fn init_global(interrupt_model: &InterruptModel) {
    match interrupt_model {
        InterruptModel::Apic(apic) => apic::set_lapic_addr(apic.local_apic_address as usize),
        _ => panic!("Unsupported interrupts mode!"),
    };
}


use core::ops::{Deref, DerefMut};
use crate::cpu_local::current_cpu;

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
        let interrupts_depth = &mut current_cpu().interrupts_depth;
        if *interrupts_depth == 0 {
            disable();
        }
        *interrupts_depth += 1;
        self.0
    }
}

impl<'a, T> DerefMut for InterruptGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let interrupts_depth = &mut current_cpu().interrupts_depth;
        if *interrupts_depth == 0 {
            disable();
        }
        *interrupts_depth += 1;
        self.0
    }
}

impl<'a, T> Drop for InterruptGuard<'a, T> {
    fn drop(&mut self) {
        let interrupts_depth = &mut current_cpu().interrupts_depth;
        *interrupts_depth -= 1;
        if *interrupts_depth == 0 {
            enable();
        }
    }
}
