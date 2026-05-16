use crate::arch::cpu::current_cpu;
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
        if cpu.interrupt_depth() == 0 { super::disable(); }
        cpu.enter_interrupt();
        self.0
    }
}

impl<'a, T> DerefMut for InterruptGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let cpu = current_cpu();
        if cpu.interrupt_depth() == 0 { super::disable(); }
        cpu.enter_interrupt();
        self.0
    }
}

impl<'a, T> Drop for InterruptGuard<'a, T> {
    fn drop(&mut self) {
        let cpu = current_cpu();
        cpu.leave_interrupt();
        if cpu.interrupt_depth() == 0 { super::enable(); }
    }
}
