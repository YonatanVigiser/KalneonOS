use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub struct InterruptSafe<T>(T);

impl<T> InterruptSafe<T> {
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    pub fn get(&self) -> InterruptSafeRef<'_, T> {
        InterruptSafeRef {
            inner: &self.0,
            _irq_guard: IrqGuard::new(),
        }
    }
    pub fn get_mut(&mut self) -> InterruptSafeMut<'_, T> {
        InterruptSafeMut {
            inner: &mut self.0,
            _irq_guard: IrqGuard::new(),
        }
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub struct IrqGuard {
    was_enabled: bool,
    _ns: PhantomData<*const ()>,
}

impl IrqGuard {
    fn new() -> Self {
        let was_enabled = super::are_enabled();
        Self {
            was_enabled,
            _ns: PhantomData,
        }
    }
}

impl Drop for IrqGuard {
    fn drop(&mut self) {
        super::set(self.was_enabled);
    }
}

pub struct InterruptSafeRef<'a, T> {
    inner: &'a T,
    _irq_guard: IrqGuard,
}

impl<T> Deref for InterruptSafeRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner
    }
}

pub struct InterruptSafeMut<'a, T> {
    inner: &'a mut T,
    _irq_guard: IrqGuard,
}

impl<T> Deref for InterruptSafeMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner
    }
}

impl<T> DerefMut for InterruptSafeMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
