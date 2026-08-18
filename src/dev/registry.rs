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

pub struct Slot<R: ?Sized> {
    entries: Vec<(DeviceId, Arc<R>)>,
    generation: u64,
}

impl<R: ?Sized> Slot<R> {
    pub const fn new() -> Self {
        Slot { entries: Vec::new(), generation: 0 }
    }

    fn push(&mut self, id: DeviceId, dev: Arc<R>) {
        self.entries.push((id, dev));
        self.generation += 1;
    }

    fn entries(&self) -> &[(DeviceId, Arc<R>)] {
        &self.entries
    }
}

impl<R: ?Sized> Default for Slot<R> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Role: 'static {
    fn slot(reg: &DeviceRegistry) -> &Slot<Self>;
    fn slot_mut(reg: &mut DeviceRegistry) -> &mut Slot<Self>;
}

macro_rules! define_registry {
    ($($field:ident : $iface:ty),+ $(,)?) => {
        pub struct DeviceRegistry {
            generation: u64,
            $($field: Slot<$iface>,)+
        }

        impl DeviceRegistry {
            const fn new() -> Self {
                DeviceRegistry {
                    generation: 0,
                    $($field: Slot::new(),)+
                }
            }

            pub fn ids(&self) -> Vec<DeviceId> {
                let mut out = Vec::new();
                $( for (id, _) in self.$field.entries() {
                    if !out.contains(id) { out.push(*id); }
                } )+
                out.sort_unstable();
                out
            }
        }

        $(
            impl Role for $iface {
                fn slot(reg: &DeviceRegistry) -> &Slot<Self> { &reg.$field }
                fn slot_mut(reg: &mut DeviceRegistry) -> &mut Slot<Self> { &mut reg.$field }
            }
        )+
    };
}

use super::traits::*;
define_registry! {
    uptime_source: dyn UptimeSource,
    log_sink: dyn LogSink,
    global_irq_controller: dyn GlobalIrqController,
    char_out: dyn CharOut,
}

impl DeviceRegistry {
    pub fn register<R: Role + ?Sized>(&mut self, dev: Arc<R>) -> DeviceId {
        let id = DeviceId::next();
        R::slot_mut(self).push(id, dev);
        self.generation += 1;
        id
    }

    pub fn add_role<R: Role + ?Sized>(&mut self, id: DeviceId, dev: Arc<R>) {
        R::slot_mut(self).push(id, dev);
        self.generation += 1;
    }

    pub fn query<R: Role + ?Sized>(&self) -> Vec<(DeviceId, Arc<R>)> {
        R::slot(self).entries().iter().cloned().collect()
    }

    pub fn get<R: Role + ?Sized>(&self, id: DeviceId) -> Option<Arc<R>> {
        R::slot(self)
            .entries()
            .iter()
            .find(|(d, _)| *d == id)
            .map(|(_, dev)| Arc::clone(dev))
    }

    pub fn generation(&self) -> u64 {
        self.generation
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
