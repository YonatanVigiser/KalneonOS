use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::rwlock::RwLock;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DeviceId(u64);

impl DeviceId {
    fn next() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone)]
pub struct DeviceInfo {
    id: DeviceId,
    lease_state: Option<Arc<LeaseState>>,
}

impl DeviceInfo {
    fn new(exclusive: bool) -> Self {
        let id = DeviceId::next();
        let lease_state = exclusive.then(|| Arc::new(LeaseState::default()));
        Self { id, lease_state }
    }
}

#[derive(Clone)]
pub struct Device<R: Role + ?Sized> {
    info: DeviceInfo,
    dev: Arc<R>,
}

impl<R: Role + ?Sized> Device<R> {
    pub fn new(info: DeviceInfo, dev: Arc<R>) -> Self {
        Self { info, dev }
    }
}

pub struct Slot<R: Role + ?Sized> {
    entries: Vec<Device<R>>,
    generation: u64,
}

impl<R: Role + ?Sized> Slot<R> {
    pub const fn new() -> Self {
        Slot { entries: Vec::new(), generation: 0 }
    }

    fn push(&mut self, dev: Device<R>) {
        self.entries.push(dev);
        self.generation += 1;
    }

    fn entries(&self) -> &[Device<R>] {
        &self.entries
    }
}

impl<R: Role + ?Sized> Default for Slot<R> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Role: 'static {
    type Access: Access;
    fn slot(reg: &DeviceRegistry) -> &Slot<Self>;
    fn slot_mut(reg: &mut DeviceRegistry) -> &mut Slot<Self>;
}

macro_rules! define_registry {
    (
        shared { $($sfield:ident : $siface:ty),* $(,)? }
        exclusive { $($xfield:ident : $xiface:ty),* $(,)? }
    ) => {
        pub struct DeviceRegistry {
            generation: u64,
            $($sfield: Slot<$siface>,)*
            $($xfield: Slot<$xiface>,)*
        }

        impl DeviceRegistry {
            const fn new() -> Self {
                DeviceRegistry {
                    generation: 0,
                    $($sfield: Slot::new(),)*
                    $($xfield: Slot::new(),)*
                }
            }

            pub fn ids(&self) -> Vec<DeviceId> {
                let mut out = Vec::new();
                $( for dev in self.$sfield.entries() {
                    if !out.contains(&dev.info.id) { out.push(dev.info.id); }
                } )*
                $( for dev in self.$xfield.entries() {
                    if !out.contains(&dev.info.id) { out.push(dev.info.id); }
                } )*
                out.sort_unstable();
                out
            }
        }

        $(
            impl Role for $siface {
                type Access = Shared;
                fn slot(reg: &DeviceRegistry) -> &Slot<Self> { &reg.$sfield }
                fn slot_mut(reg: &mut DeviceRegistry) -> &mut Slot<Self> { &mut reg.$sfield }
            }
        )*
        $(
            impl Role for $xiface {
                type Access = Exclusive;
                fn slot(reg: &DeviceRegistry) -> &Slot<Self> { &reg.$xfield }
                fn slot_mut(reg: &mut DeviceRegistry) -> &mut Slot<Self> { &mut reg.$xfield }
            }
        )*
    };
}

use crate::drivers::input::{InputEvent, KeyEvent, KeyboardDevice};
use crate::interrupt::{GlobalInterruptController, LocalInterruptController};

use super::lease::{Access, Acquire, Exclusive, Lease, LeaseState, OnReleaseRequest, Shared};
use super::traits::*;
define_registry! {
    shared {
        uptime_source: dyn UptimeSource,
        global_interrupt_controller: dyn GlobalInterruptController,
        local_interrupt_controller: dyn LocalInterruptController,
        keyboard: dyn KeyboardDevice,
        keyboard_input_event: dyn InputEvent<KeyEvent>,
    }
    exclusive {
        log_sink: dyn LogSink,
        char_out: dyn CharOut,
    }
}

impl DeviceRegistry {
    pub fn register<R: Role + ?Sized>(&mut self, dev: Arc<R>) -> DeviceInfo {
        let dev = Device::new(DeviceInfo::new(R::Access::EXLUSIVE), dev);
        let info = dev.info.clone();
        R::slot_mut(self).push(dev);
        self.generation += 1;
        info
    }

    pub fn add_role<R: Role + ?Sized>(&mut self, dev: Device<R>) {
        assert_eq!(dev.info.lease_state.is_some(), R::Access::EXLUSIVE, "Added Role doesn't match internal lease state");
        R::slot_mut(self).push(dev);
        self.generation += 1;
    }

    pub fn ids_with_role<R: Role + ?Sized>(&self) -> Vec<DeviceId> {
        R::slot(self).entries().iter().map(|dev| dev.info.id).collect()
    }

    pub fn get<R>(&self, id: DeviceId) -> Option<Arc<R>>
    where R: Role<Access = Shared> + ?Sized {
        R::slot(self).entries().iter().find(|dev| dev.info.id == id).map(|dev| dev.dev.clone())
    }

    pub fn query<R>(&self) -> Vec<Arc<R>>
    where R: Role<Access = Shared> + ?Sized {
        R::slot(self).entries().iter().map(|dev| dev.dev.clone()).collect()
    }

    pub fn try_acquire<R>(&self, id: DeviceId, callback: Option<OnReleaseRequest>) -> Option<Lease<R>>
    where R: Role<Access = Exclusive> + ?Sized {
        let dev = R::slot(self).entries().iter().find(|dev| dev.info.id == id)?;
        dev.info.lease_state.as_ref().unwrap().try_acquire(callback).then(|| Lease::<R>::new(dev.dev.clone(), dev.info.lease_state.clone().unwrap()))
    }

    pub fn try_acquire_all<R>(&self) -> Vec<Lease<R>>
    where R: Role<Access = Exclusive> + ?Sized {
    }

    pub fn acquire<R>(&self, id: DeviceId, callback: Option<OnReleaseRequest>) -> Option<Acquire<R>>
    where R: Role<Access = Exclusive> + ?Sized {
        let dev = R::slot(self).entries().iter().find(|dev| dev.info.id == id)?;
        Some(Acquire::new(dev.dev.clone(), dev.info.lease_state.as_ref().unwrap().clone(), callback))
    }

    pub fn request_release<R>(&self, id: DeviceId)
        where R: Role<Access = Exclusive> + ?Sized {
        if let Some(dev) = R::slot(self).entries().iter().find(|dev| dev.info.id == id) {
            dev.info.lease_state.as_ref().unwrap().request_release();
        }
    }

    pub fn slot_generation<R: Role + ?Sized>(&self) -> u64 {
        R::slot(self).generation
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub static DEVICE_REGISTRY: RwLock<DeviceRegistry> = RwLock::new(DeviceRegistry::new());
