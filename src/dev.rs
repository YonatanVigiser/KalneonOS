pub mod registry;

use core::{any::{Any, TypeId}, task::{Context, Poll}};

use alloc::{boxed::Box, sync::Arc, vec::Vec};

pub trait Read<T> {
    fn read(&self, cx: &mut Context) -> Poll<Result<T, DeviceError>>;
}

pub trait Write<T> {
    fn write(&self, cx: &mut Context, value: &mut Option<T>) -> Poll<Result<(), DeviceError>>;
}

pub trait ReadAt<I, T> {
    fn read_at(&self, cx: &mut Context, index: &I) -> Poll<Result<T, DeviceError>>;
}

pub trait WriteAt<I, T> {
    fn write_at(&self, cx: &mut Context, index: &I, value: &mut Option<T>) -> Poll<Result<(), DeviceError>>;
}

pub trait Info<T> {
    fn info(&self) -> T;
}

pub enum DeviceError {
    Disconnected,
    Hardware,
    Unsupported,
    OutOfBounds,
}

type ErasedDevice = Box<dyn Any + Send + Sync>;

pub trait Device: Send + Sync + 'static {
    fn capabilities(self: Arc<Self>) -> Vec<(TypeId, ErasedDevice)>;
}

