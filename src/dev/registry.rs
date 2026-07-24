use core::any::TypeId;

use alloc::{collections::{btree_map::BTreeMap, btree_set::BTreeSet}, sync::{Arc, Weak}, vec::Vec};

use crate::dev::{Device, ErasedDevice};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DeviceId(u64);

#[macro_export]
macro_rules! device_caps {
    ($ty:ty : $($iface:ty),+ $(,)?) => {
        impl $crate::dev::Device for $ty {
            fn capabilities(
                self: ::alloc::sync::Arc<Self>,
            ) -> ::alloc::vec::Vec<(::core::any::TypeId, $crate::dev::ErasedDevice)> {
                ::alloc::vec![
                    $((
                        ::core::any::TypeId::of::<$iface>(),
                        ::alloc::boxed::Box::new(::alloc::sync::Arc::downgrade( &(::alloc::sync::Arc::clone(&self) as ::alloc::sync::Arc<$iface>),
                        )) as $crate::dev::ErasedDevice,
                    ),)+
                ]
            }
        }
    };
}
 
#[macro_export]
macro_rules! allow {
    ($($iface:ty),* $(,)?) => {{
        let mut set: ::alloc::collections::BTreeSet<::core::any::TypeId> =
            ::alloc::collections::BTreeSet::new();
        $( set.insert(::core::any::TypeId::of::<$iface>()); )*
        set
    }};
}
 
struct Registration {
    anchor: Option<Arc<dyn Device>>, // Represents device ownership, keeps the device alive
    allowed: BTreeMap<TypeId, ErasedDevice>,
}
 
pub struct DeviceRegistry {
    next_id: u64,
    devices: BTreeMap<DeviceId, Registration>,
    index: BTreeMap<TypeId, Vec<DeviceId>>,
}
 
impl DeviceRegistry {
    pub const fn new() -> Self {
        DeviceRegistry {
            next_id: 0,
            devices: BTreeMap::new(),
            index: BTreeMap::new(),
        }
    }
 
    fn next_id(&mut self) -> DeviceId {
        let id = DeviceId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn register<D: Device>(&mut self, dev: D, allow: BTreeSet<TypeId>) -> DeviceId {
        let anchor: Arc<D> = Arc::new(dev);
        let caps = Arc::clone(&anchor).capabilities();
        let id = self.next_id();
 
        let mut allowed = BTreeMap::new();
        for (tid, handle) in caps {
            if allow.contains(&tid) && !allowed.contains_key(&tid) {
                allowed.insert(tid, handle);
                self.index.entry(tid).or_default().push(id);
            }
        }
 
        self.devices.insert(
            id,
            Registration {
                anchor: Some(anchor as Arc<dyn Device>),
                allowed,
            },
        );
        id
    }

    pub fn borrow(
        &mut self,
        from: &DeviceRegistry,
        id: DeviceId,
        allow: &BTreeSet<TypeId>,
    ) -> Option<DeviceId> {
        let src = from.devices.get(&id)?;
        let anchor = src.anchor.as_ref()?;
 
        let caps = Arc::clone(anchor).capabilities();
 
        let mut allowed = BTreeMap::new();
        for (tid, handle) in caps {
            if allow.contains(&tid)
                && src.allowed.contains_key(&tid)
                && !allowed.contains_key(&tid)
            {
                allowed.insert(tid, handle);
            }
        }
 
        let new_id = self.next_id();
        for tid in allowed.keys() {
            self.index.entry(*tid).or_default().push(new_id);
        }
        self.devices.insert(
            new_id,
            Registration {
                anchor: None,
                allowed,
            },
        );
        Some(new_id)
    }

    pub fn get<T: ?Sized + 'static>(&self, id: DeviceId) -> Option<Arc<T>> {
        let tid = TypeId::of::<T>();
        let reg = self.devices.get(&id)?;
        let erased = reg.allowed.get(&tid)?;
        let weak = erased.downcast_ref::<Weak<T>>()?;
        Some(weak.upgrade()?)
    }

    pub fn find<T: ?Sized + 'static>(&self) -> Vec<DeviceId> {
        let tid = TypeId::of::<T>();
        let mut out = Vec::new();
        let Some(ids) = self.index.get(&tid) else {
            return out;
        };
        for id in ids {
            if let Some(_) = self.get::<T>(*id) {
                out.push(*id);
            }
        }
        out
    }
 
    pub fn query<T: ?Sized + 'static>(&self) -> Vec<Arc<T>> {
        let tid = TypeId::of::<T>();
        let mut out = Vec::new();
        let Some(ids) = self.index.get(&tid) else {
            return out;
        };
        for id in ids {
            if let Some(dev) = self.get(*id) {
                out.push(dev);
            }
        }
        out
    }

    pub fn remove(&mut self, id: DeviceId) -> bool {
        let Some(reg) = self.devices.remove(&id) else {
            return false;
        };
        for tid in reg.allowed.keys() {
            let now_empty = match self.index.get_mut(tid) {
                Some(v) => {
                    v.retain(|d| *d != id);
                    v.is_empty()
                }
                None => false,
            };
            if now_empty {
                self.index.remove(tid);
            }
        }
        true
    }
}
 
impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
