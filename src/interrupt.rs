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
use core::fmt::{Display};
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
pub struct LocalInterruptSource(pub u32);

#[derive(Debug, Default)]
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
    SourceReserved(LocalInterruptSource),
    AlreadyAllocated(LocalInterruptSource),
    MaskingNotSupported,
}

#[derive(Debug, Default, Clone)]
pub enum InterruptSourceState {
    #[default]
    Free,
    Reserved(Arc<InterruptSlot>),
    Allocated(Arc<InterruptSlot>),
}

impl InterruptSourceState {
    pub fn slot(&self) -> Option<&Arc<InterruptSlot>> {
        match self {
            Self::Free => None,
            Self::Reserved(s) | Self::Allocated(s) => Some(s),
        }
    }
}

pub trait LocalInterruptController: Send + Sync {
    fn allocate_local_source(
        &self,
    ) -> Result<(LocalInterruptSource, Arc<InterruptSlot>), LocalInterruptControllerError>;
    fn free_local_source(&self, source: LocalInterruptSource) -> Result<(), LocalInterruptControllerError>;
    fn reserve(&self, source: LocalInterruptSource) -> Result<Arc<InterruptSlot>, LocalInterruptControllerError>;
    fn slot(&self, source: LocalInterruptSource) -> Option<Arc<InterruptSlot>>;
    fn state(&self, source: LocalInterruptSource) -> InterruptSourceState;
    fn available_local_sources_count(&self) -> usize;
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
    fn interrupt_destination_id(&self) -> usize;
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
    pub fn slot(&self) -> &Arc<InterruptSlot> {
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
    UnknownTarget(CpuId),
}

pub trait GlobalInterruptController: Send + Sync {
    fn add_local_interrupt_controller(
        &self,
        local_interrupt_controller: Arc<dyn LocalInterruptController>,
    );
    fn allocate_target(
        &self,
    ) -> Result<(LocalInterruptTarget, Arc<InterruptSlot>), GlobalInterruptControllerError>;
    /// Routes `source` to `target`.
    ///
    /// The line is left **masked**. Call [`unmask`] once a listener is
    /// registered on the target's slot — otherwise the first interrupt
    /// arrives before anyone is waiting for it.
    ///
    /// Fails with [`NonMaskableInterrupt`] if the platform has configured
    /// this source as an NMI.
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

impl Display for LocalInterruptControllerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidLIS(s) => write!(f, "invalid local interrupt source {}", s.0),
            Self::OutOfSources => f.write_str("no free local interrupt sources"),
            Self::SourceNotAllocated => f.write_str("local interrupt source is not allocated"),
            Self::SourceReserved(s) => write!(f, "local interrupt source {} is reserved", s.0),
            Self::AlreadyAllocated(s) => write!(f, "local interrupt source {} is already in use", s.0),
            Self::MaskingNotSupported => f.write_str("this controller cannot mask local sources"),
        }
    }
}

impl Error for LocalInterruptControllerError {}

impl Display for GlobalInterruptControllerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidGIS(s) => write!(f, "invalid global interrupt source {}", s.0),
            Self::NoLocalInterruptControllers => f.write_str("no local interrupt controllers registered"),
            Self::LocalInterruptControllerError(e) => write!(f, "local controller: {e}"),
            Self::OutOfLocalSources => f.write_str("no local interrupt controller has a free source"),
            Self::NonMaskableInterrupt(s) => write!(f, "global interrupt source {} is non-maskable", s.0),
            Self::AlreadyRouted(t) => write!(
                f, "already routed to cpu {:?} source {}", t.cpu_id, t.local_source.0
            ),
            Self::NotRouted(s) => write!(f, "global interrupt source {} is not routed", s.0),
            Self::UnknownTarget(id) => write!(f, "no local interrupt controller for the following {:?}", id),
        }
    }
}

impl Error for GlobalInterruptControllerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LocalInterruptControllerError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<LocalInterruptControllerError> for GlobalInterruptControllerError {
    fn from(e: LocalInterruptControllerError) -> Self {
        Self::LocalInterruptControllerError(e)
    }
}
