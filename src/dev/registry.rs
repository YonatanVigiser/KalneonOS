use core::any::TypeId;

use alloc::{collections::{btree_map::BTreeMap, btree_set::BTreeSet}, sync::{Arc, Weak}, vec::Vec};

use crate::dev::{Device, ErasedDevice};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DeviceId(u64);

#[macro_export]
macro_rules! device_caps {
    ($ty:ty : $($iface:ty),+ $(,)?) => {
        impl $crate::Device for $ty {
            fn capabilities(
                self: ::alloc::sync::Arc<Self>,
            ) -> ::alloc::vec::Vec<(::core::any::TypeId, $crate::Erased)> {
                ::alloc::vec![
                    $((
                        ::core::any::TypeId::of::<$iface>(),
                        ::alloc::boxed::Box::new(::alloc::sync::Arc::downgrade(
                            &(::alloc::sync::Arc::clone(&self) as ::alloc::sync::Arc<$iface>),
                        )) as $crate::Erased,
                    ),)+
                ]
            }
        }
    };
}
 
#[macro_export]
macro_rules! ifaces {
    ($($iface:ty),* $(,)?) => {{
        let mut set: ::alloc::collections::BTreeSet<::core::any::TypeId> =
            ::alloc::collections::BTreeSet::new();
        $( set.insert(::core::any::TypeId::of::<$iface>()); )*
        set
    }};
}
 
struct Registration {
    anchor: Option<Arc<dyn Device>>,
    allowed: BTreeMap<TypeId, ErasedDevice>,
}
 
pub struct Registry {
    next_id: u64,
    devices: BTreeMap<DeviceId, Registration>,
    index: BTreeMap<TypeId, Vec<DeviceId>>,
}
 
impl Registry {
    pub const fn new() -> Self {
        Registry {
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

    pub fn register<D: Device>(&mut self, dev: D, allow: &BTreeSet<TypeId>) -> DeviceId {
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
        from: &Registry,
        id: DeviceId,
        allow: &BTreeSet<TypeId>,
    ) -> Option<DeviceId> {
        let src = from.devices.get(&id)?;
        let anchor = src.anchor.as_ref()?; // None ⇒ `from` is itself a borrower
 
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
 
    pub fn query<T: ?Sized + 'static>(&self) -> Vec<Arc<T>> {
        let tid = TypeId::of::<T>();
        let mut out = Vec::new();
        let Some(ids) = self.index.get(&tid) else {
            return out;
        };
        for id in ids {
            let Some(reg) = self.devices.get(id) else {
                continue;
            };
            let Some(erased) = reg.allowed.get(&tid) else {
                continue;
            };
            let Some(weak) = erased.downcast_ref::<Weak<T>>() else {
                continue; // unreachable if handles were built by device_caps!
            };
            if let Some(strong) = weak.upgrade() {
                out.push(strong);
            }
        }
        out
    }
 
    pub fn with_each<T: ?Sized + 'static>(&self, mut f: impl FnMut(&T)) {
        let tid = TypeId::of::<T>();
        let Some(ids) = self.index.get(&tid) else {
            return;
        };
        for id in ids {
            let strong = self
                .devices
                .get(id)
                .and_then(|reg| reg.allowed.get(&tid))
                .and_then(|erased| erased.downcast_ref::<Weak<T>>())
                .and_then(|weak| weak.upgrade());
            if let Some(strong) = strong {
                f(&strong); // the Arc lives only for this call frame
            }
        }
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
 
impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}
 

