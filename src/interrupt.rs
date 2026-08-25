pub mod apic;
pub mod guard;
pub mod handlers;
pub mod mutex;

pub const SPURIOUS_VECTOR: u8 = 0xFF;
pub const CONTROLLER_ERROR_VECTOR: u8 = 0xFE;
pub const TIMER_IRQ_VECTOR: u8 = 0x30;

pub fn init_global(interrupts_model: &InterruptModel) {
    match interrupts_model {
        InterruptModel::Apic(apic_info) => apic::init_global(apic_info.clone()),
        _ => panic!("Unsupported interrupts model"),
    };
}

pub fn init_local() -> LocalApic {
    apic::init_lapic()
}

pub fn register_local(lapic: LocalApic) {
    let lapic_dev = Arc::new(LocalApicDevice::new(lapic, current_cpu().logical_id));
    current_cpu().lapic = Some(lapic_dev.clone());
    DEVICE_REGISTRY
        .read()
        .query::<dyn GlobalInterruptController>()
        .first()
        .expect("No GlobalInterruptController")
        .1
        .add_local_interrupt_controller(lapic_dev);
}

#[inline(always)]
pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

#[inline(always)]
pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

#[inline(always)]
pub fn set(enabled: bool) {
    if enabled {
        enable()
    }
}

#[inline(always)]
pub fn are_enabled() -> bool {
    x86_64::instructions::interrupts::are_enabled()
}

use core::error::Error;
use core::fmt::Display;
use core::future::poll_fn;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use core::task::{Context, Poll};

use acpi::platform::InterruptModel;
use alloc::sync::Arc;
use alloc::vec::Vec;
use futures::task::AtomicWaker;
use x2apic::lapic::LocalApic;

use crate::arch::cpu::{CpuId, current_cpu};
use crate::dev::registry::DEVICE_REGISTRY;

use self::apic::LocalApicDevice;
use self::mutex::InterruptSafeMutex;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalInterruptSource(u32);

impl LocalInterruptSource {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Default)]
pub struct InterruptSlot {
    count: AtomicUsize,
    consumers: InterruptSafeMutex<Vec<Arc<InterruptEvent>>>,
}

impl InterruptSlot {
    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    fn dispatch(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
        let consumers = self.consumers.lock();
        for event in consumers.iter() {
            event.signal();
        }
    }

    pub fn listen(self: &Arc<Self>) -> InterruptListener {
        let event = Arc::new(InterruptEvent::default());
        self.consumers.lock().push(Arc::clone(&event));
        InterruptListener {
            slot: Arc::clone(self),
            event,
            last_acked: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalInterruptControllerError {
    InvalidLIS(LocalInterruptSource),
    OutOfSources,
    SourceNotAllocated,
    MaskingNotSupported,
}

impl Error for LocalInterruptControllerError {}

impl Display for LocalInterruptControllerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Local Interrupt Controller Error: {:?}", self)
    }
}

pub trait LocalInterruptController: Send + Sync {
    fn allocate_local_source(
        &self,
    ) -> Result<(LocalInterruptSource, Arc<InterruptSlot>), LocalInterruptControllerError>;
    fn free_local_source(&self, source: LocalInterruptSource) -> Result<Arc<InterruptSlot>, LocalInterruptControllerError>;
    fn slot(&self, source: LocalInterruptSource) -> Option<Arc<InterruptSlot>>;
    fn avaible_local_sources_count(&self) -> usize;
    fn enter_interrupt(
        &self,
        source: LocalInterruptSource,
    ) -> Result<(), LocalInterruptControllerError>;
    fn enter_spurious_interrupt(&self);
    fn mask(&self, source: LocalInterruptSource) -> Result<(), LocalInterruptControllerError>;
    fn unmask(&self, source: LocalInterruptSource) -> Result<(), LocalInterruptControllerError>;
    fn can_mask(&self, source: LocalInterruptSource) -> bool;
    fn interrupts_count(&self) -> usize;
    fn spurious_interrupts_count(&self) -> usize;
    fn cpu_id(&self) -> CpuId;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalInterruptTarget {
    pub cpu_id: CpuId,
    pub local_source: LocalInterruptSource,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalInterruptSource(pub u32);

#[derive(Debug, Default)]
struct InterruptEvent {
    seq: AtomicU32,
    waker: AtomicWaker,
}

pub struct InterruptListener {
    slot: Arc<InterruptSlot>,
    event: Arc<InterruptEvent>,
    last_acked: u32,
}

impl InterruptEvent {
    fn signal(&self) {
        self.seq.fetch_add(1, Ordering::Release);
        self.waker.wake();
    }
}

impl InterruptListener {
    fn slot(&self) -> &Arc<InterruptSlot> {
        &self.slot
    }

    pub fn poll_wait(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        let current = self.event.seq.load(Ordering::Acquire);
        if current != self.last_acked {
            self.last_acked = current;
            Poll::Ready(())
        } else {
            self.event.waker.register(cx.waker());
            let current = self.event.seq.load(Ordering::Acquire);
            if current != self.last_acked {
                self.last_acked = current;
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }

    pub async fn wait(&mut self) {
        poll_fn(|cx| self.poll_wait(cx)).await
    }

    pub async fn wait_until<T>(&mut self, mut f: impl FnMut() -> Option<T>) -> T {
        loop {
            if let Some(v) = f() {
                return v;
            }
            self.wait().await;
        }
    }
}

impl Drop for InterruptListener {
    fn drop(&mut self) {
        let mut consumers = self.slot.consumers.lock();
        if let Some(i) = consumers.iter().position(|e| Arc::ptr_eq(e, &self.event)) {
            consumers.swap_remove(i);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalInterruptControllerError {
    InvalidGIS(GlobalInterruptSource),
    NoLocalInterruptControllers,
    LocalInterruptControllerError(LocalInterruptControllerError),
    OutOfLocalSources,
    NonMaskableInterrupt(GlobalInterruptSource),
    AlreadyRouted(LocalInterruptTarget),
    NotRouted(GlobalInterruptSource),
}

impl Error for GlobalInterruptControllerError {}

impl Display for GlobalInterruptControllerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Global Interrupt Controller Error: {:?}", self)
    }
}

pub trait GlobalInterruptController: Send + Sync {
    fn add_local_interrupt_controller(
        &self,
        local_interrupt_controller: Arc<dyn LocalInterruptController>,
    );
    fn allocate_target(
        &self,
    ) -> Result<(LocalInterruptTarget, Arc<InterruptSlot>), GlobalInterruptControllerError>;
    fn route(
        &self,
        source: GlobalInterruptSource,
        target: LocalInterruptTarget,
    ) -> Result<(), GlobalInterruptControllerError>;
    fn unroute(&self, source: GlobalInterruptSource) -> Result<(), GlobalInterruptControllerError>;
    fn mask(&self, source: GlobalInterruptSource) -> Result<(), GlobalInterruptControllerError>;
    fn unmask(&self, source: GlobalInterruptSource) -> Result<(), GlobalInterruptControllerError>;
    fn routed_lines(&self) -> Vec<GlobalInterruptSource>;
    fn interrupt_target(&self, gis: GlobalInterruptSource) -> Option<LocalInterruptTarget>;
    fn interrupt_count(&self, gis: GlobalInterruptSource) -> Option<usize>;
    fn total_interrupt_count(&self) -> usize;
}
