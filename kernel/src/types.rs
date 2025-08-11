use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct SyncUnsafeCell<T>(pub UnsafeCell<T>);

unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
  pub fn get_mut(&self) -> &mut T {
    unsafe { &mut *self.0.get() }
  }
}
