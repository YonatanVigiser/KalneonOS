pub mod handlers;
pub mod guard;
pub mod mutex;
mod apic;

pub const SPURIOUS_VECTOR: u8 = 0xFF;
pub const CONTROLLER_ERROR_VECTOR: u8 = 0xFE;
pub const TIMER_VECOTR: u8 = 0x30;

pub fn init_global(interrupts_model: &InterruptModel) {
    match interrupts_model {
        InterruptModel::Apic(apic_info) => apic::init_global(apic_info.clone()),
        _ => panic!("Unsupported interrupts model"),
    };
}

pub fn init_local() -> LocalApic {
    let mut lapic = apic::init_lapic();
    apic::init_lapic_timer(&mut lapic, 1000000);
    lapic
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

pub fn set(enabled: bool) {
    if enabled { enable() }
}

pub fn are_enabled() -> bool {
    x86_64::instructions::interrupts::are_enabled()
}

use core::error::Error;
use core::fmt::Display;
use core::future::poll_fn;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use core::task::{Context, Poll};

use acpi::platform::InterruptModel;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use futures::task::AtomicWaker;
use x2apic::lapic::LocalApic;

use self::mutex::InterruptSafeMutex;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct LocalInterruptSource(u32);

#[derive(Default)]
pub struct IrqSlot {
    count: AtomicUsize,
    consumers: InterruptSafeMutex<Vec<Weak<IrqEvent>>>,
}

impl IrqSlot {
    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    fn dispatch(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
        let consumers = self.consumers.lock();
        for event in consumers.iter() {
            if let Some(event) = event.upgrade() {
                event.signal();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LICError {
    OutOfVectors,
    NoIrqSlotRegistered,
    NotAnIrqVector
}

impl Error for LICError {}

impl Display for LICError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Local Interrupt Controller Error: {:?}", self)
    }
}

pub trait LocalInterruptController: Send + Sync {
    fn bind(&self, gsi: GlobalInterruptSource) -> Result<(LocalInterruptSource, Arc<IrqSlot>), LICError>;
    fn get_irq_slot(&self, source: LocalInterruptSource) -> Result<Arc<IrqSlot>, LICError>;
    fn free_vectors(&self) -> usize;
    fn enter_irq(&self, source: LocalInterruptSource) -> Result<(), LICError>;
    fn enter_spurious_interrupt(&self);
    fn spurious_interrupts_count(&self) -> u64;
    fn mask(&self, source: LocalInterruptSource) -> Result<(), LICError>;
    fn unmask(&self, source: LocalInterruptSource) -> Result<(), LICError>;
    fn irq_count(&self) -> u64;
    fn cpu_id(&self) -> usize;
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalInterruptSource(u32);

pub struct IrqEvent {
    seq: AtomicU32,
    last_acked: AtomicU32,
    waker: AtomicWaker,
}

impl IrqEvent {
    fn signal(&self) {
        self.seq.fetch_add(1, Ordering::Release);
        self.waker.wake();
    }

    fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<()> {
        let current = self.seq.load(Ordering::Acquire);
        if current != self.last_acked.load(Ordering::Relaxed) {
            self.last_acked.store(current, Ordering::Release);
            Poll::Ready(())
        } else {
            self.waker.register(cx.waker());
            let current = self.seq.load(Ordering::Acquire);
            if current != self.last_acked.load(Ordering::Relaxed) {
                self.last_acked.store(current, Ordering::Release);
                Poll::Ready(())
            } else { Poll::Pending }
        }
    }

    pub async fn wait(&self) {
        poll_fn(|cx| self.poll_wait(cx)).await
    }

    pub async fn wait_until<T>(&self, mut f: impl FnMut() -> Option<T>) -> T {
        loop {
            if let Some(v) = f() { return v; }
            self.wait().await;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GICError {
    InvaildGis(GlobalInterruptSource),
    NoLocalInterruptController,
}

impl Error for GICError {}

impl Display for GICError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Global Interrupt Controller Error: {:?}", self)
    }
}

pub trait GlobalInterruptController: Send + Sync {
    fn add_local_interrupt_controller(&self, local_interrupt_controller: Arc<dyn LocalInterruptController>);
    fn register_event(&self, source: GlobalInterruptSource, event: Weak<IrqEvent>) -> Result<(), GICError>;
    fn gis_count(&self, source: GlobalInterruptSource) -> u64;
    fn total_gis_count(&self) -> u64;
    fn mask(&self ,source: GlobalInterruptSource);
    fn unmask(&self, source: GlobalInterruptSource);
}

