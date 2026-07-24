pub mod registry;

use core::{any::{Any, TypeId}, task::{Context, Poll}};

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::dev::registry::DeviceRegistry;

pub trait Read<T>: Device {
    fn read(&self, cx: &mut Context) -> Poll<Result<T, DeviceError>>;
}

pub trait ReadSync<T>: Device {
    fn read_sync(&self) -> Result<T, DeviceError>;
}

impl<T, D: ReadSync<T>> Read<T> for D {
    fn read(&self, _cx: &mut Context) -> Poll<Result<T, DeviceError>> {
        Poll::Ready(self.read_sync())
    }
}

pub trait Write<T>: Device {
    fn write(&self, cx: &mut Context, value: &mut Option<T>) -> Poll<Result<(), DeviceError>>;
}

pub trait WriteSync<T>: Device {
    fn write_sync(&self, value: T) -> Result<(), DeviceError>;
}

impl<T, D: WriteSync<T>> Write<T> for D {
    fn write(&self, _cx: &mut Context, value: &mut Option<T>) -> Poll<Result<(), DeviceError>> {
        let result = self.write_sync(value.take().ok_or(DeviceError::EmptyWrite)?);
        Poll::Ready(result)
    }
}

pub trait ReadAt<I, T>: Device {
    fn read_at(&self, cx: &mut Context, index: &I) -> Poll<Result<T, DeviceError>>;
}

pub trait ReadAtSync<I, T>: Device {
    fn read_at_sync(&self, index: &I) -> Result<T, DeviceError>;
}

impl<I, T, D: ReadAtSync<I, T>> ReadAt<I, T> for D {
    fn read_at(&self, _cx: &mut Context, index: &I) -> Poll<Result<T, DeviceError>> {
        Poll::Ready(self.read_at_sync(index))
    }
}

pub trait WriteAt<I, T>: Device {
    fn write_at(&self, cx: &mut Context, index: &I, value: &mut Option<T>) -> Poll<Result<(), DeviceError>>;
}

pub trait WriteAtSync<I, T>: Device {
    fn write_at_sync(&self, index: &I, value: T) -> Result<(), DeviceError>;
}

impl<I, T, D: WriteAtSync<I, T>> WriteAt<I, T> for D {
    fn write_at(&self, _cx: &mut Context, index: &I, value: &mut Option<T>) -> Poll<Result<(), DeviceError>> {
        let result = self.write_at_sync(index, value.take().ok_or(DeviceError::EmptyWrite)?);
        Poll::Ready(result)
    }
}

pub trait Info<T>: Device {
    fn info(&self) -> T;
}

#[derive(Debug)]
pub enum DeviceError {
    Disconnected,
    Hardware,
    Unsupported,
    OutOfBounds,
    EmptyWrite,
}

pub type ErasedDevice = Box<dyn Any + Send + Sync>;

pub trait Device: Send + Sync + 'static {
    fn capabilities(self: Arc<Self>) -> Vec<(TypeId, ErasedDevice)>;
}

lazy_static! {
    pub static ref GLOBAL_REGISTRY: Arc<Mutex<DeviceRegistry>> = Arc::new(Mutex::new(DeviceRegistry::new()));
}
