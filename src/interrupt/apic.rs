use acpi::platform::interrupt::{Apic, InterruptSourceOverride, IoApic as IoApicInfo};
use acpi::sdt::madt::{Polarity, TriggerMode};
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::structures::paging::PageSize;

use crate::arch::cpu::{CpuId, current_cpu};
use crate::dev::registry::{DEVICE_REGISTRY};
use crate::interrupt::{CONTROLLER_ERROR_VECTOR, InterruptSlot, InterruptSourceState, SPURIOUS_VECTOR, TIMER_IRQ_VECTOR};
use crate::memory::FrameSize;
use crate::{memory::map_mmio_ptr, time::KernelDuration};

use super::mutex::InterruptSafeMutex;
use super::{
    GlobalInterruptController, GlobalInterruptControllerError, GlobalInterruptSource,
    LocalInterruptController, LocalInterruptControllerError, LocalInterruptSource,
    LocalInterruptTarget,
};

const LAPIC_MMIO_SIZE: usize = 0x1000;

pub const IRQ_BASE: u8 = 0x30;

static LAPIC_PTR: AtomicUsize = AtomicUsize::new(0);

struct IoApicEntry {
    info: IoApicInfo,
    io_apic: IoApic,
    max_entry: u8,
}

struct ChainedIoApics {
    io_apics: Vec<IoApicEntry>,
    iso: Vec<InterruptSourceOverride>,
}

struct IoApicDevice {
    device: Mutex<ChainedIoApics>,
    local_interrupt_controllers: Mutex<Vec<Arc<dyn LocalInterruptController>>>,
    bindings: Mutex<BTreeMap<GlobalInterruptSource, LocalInterruptTarget>>,
}

impl ChainedIoApics {
    fn new(info: Apic) -> Self {
        let mut io_apics = Vec::new();
        for io_apic_info in info.io_apics {
            let mmio_addr = map_mmio_ptr(io_apic_info.address as usize, FrameSize::SIZE as usize)
                .expect("MMIO mapping failed") as u64;
            let mut io_apic = unsafe { IoApic::new(mmio_addr) };
            let max_entry = unsafe { io_apic.max_table_entry() };
            for irq in 0..=max_entry {
                let mut entry = RedirectionTableEntry::default();
                entry.set_mode(IrqMode::Fixed);
                entry.set_flags(IrqFlags::MASKED);
                unsafe {
                    io_apic.set_table_entry(irq, entry);
                }
            }
            io_apics.push(IoApicEntry {
                info: io_apic_info,
                io_apic,
                max_entry,
            });
        }
        let mut chained_io_apics = Self {
            io_apics,
            iso: info.interrupt_source_overrides,
        };
        for nmi_source in info.nmi_sources {
            let io_apic_entry = chained_io_apics
                .get_ioapic_entry(nmi_source.global_system_interrupt)
                .expect("IOAPIC with corresponding GSI number doesn't exist");
            let irq = Self::irq_from_gsi(nmi_source.global_system_interrupt, &io_apic_entry.info);
            let mut entry = unsafe { io_apic_entry.io_apic.table_entry(irq) };
            entry.set_mode(IrqMode::NonMaskable);
            let flags = flags_from(nmi_source.polarity, nmi_source.trigger_mode);
            entry.set_flags(flags);
            unsafe {
                io_apic_entry.io_apic.set_table_entry(irq, entry);
            }
        }
        chained_io_apics
    }

    fn get_ioapic_entry(&mut self, gsi: u32) -> Option<&mut IoApicEntry> {
        self.io_apics.iter_mut().find(|e| {
            let base = e.info.global_system_interrupt_base;
            gsi >= base && gsi <= base + e.max_entry as u32
        })
    }

    fn set_irq(&mut self, gsi: u32, vector: u8, dest: u8, flags: IrqFlags) {
        let io_apic_entry = self
            .get_ioapic_entry(gsi)
            .expect("IOAPIC with corresponding GSI number doesn't exist");
        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_vector(vector);
        entry.set_dest(dest);
        entry.set_flags(flags | IrqFlags::MASKED);
        unsafe {
            io_apic_entry.io_apic.set_table_entry(
                Self::irq_from_gsi(gsi, &io_apic_entry.info),
                entry,
            );
        }
    }

    fn clear_irq(&mut self, gsi: u32) {
        let io_apic_entry = self
            .get_ioapic_entry(gsi)
            .expect("IOAPIC with corresponding GSI number doesn't exist");
        let mut entry = RedirectionTableEntry::default();
        entry.set_flags(IrqFlags::MASKED);
        unsafe {
            io_apic_entry.io_apic.set_table_entry(
                Self::irq_from_gsi(gsi, &io_apic_entry.info),
                entry,
            );
        }
    }

    fn set_masked(&mut self, gsi: u32, masked: bool) {
        let io_apic_entry = self
            .get_ioapic_entry(gsi)
            .expect("IOAPIC with corresponding GSI number doesn't exist");
        unsafe {
            if masked {
                io_apic_entry.io_apic.disable_irq(Self::irq_from_gsi(gsi, &io_apic_entry.info));
            } else {
                io_apic_entry.io_apic.enable_irq(Self::irq_from_gsi(gsi, &io_apic_entry.info));
            }
        }
    }

    fn has_gsi(&self, gsi: u32) -> bool {
        self.io_apics.iter().any(|e| {
            let base = e.info.global_system_interrupt_base;
            gsi >= base && gsi <= base + e.max_entry as u32
        })
    }

    fn flags_for_gsi(&self, gsi: u32) -> IrqFlags {
        match self.iso.iter().find(|o| o.global_system_interrupt == gsi) {
            Some(o) => flags_from(o.polarity, o.trigger_mode),
            None => IrqFlags::empty(),
        }
    }

    fn can_mask(&mut self, gsi: u32) -> bool {
        let io_apic_entry = self
            .get_ioapic_entry(gsi)
            .expect("IOAPIC with corresponding GSI number doesn't exist");
        !matches!(unsafe { io_apic_entry.io_apic.table_entry(Self::irq_from_gsi(gsi, &io_apic_entry.info)) }.mode(), IrqMode::NonMaskable)
    }

    fn irq_from_gsi(gsi: u32, ioapic_info: &IoApicInfo) -> u8 {
        (gsi - ioapic_info.global_system_interrupt_base) as u8
    }
}

impl IoApicDevice {
    fn select_local_interrupt_controller(
        &self,
    ) -> Result<Arc<dyn LocalInterruptController>, GlobalInterruptControllerError> {
        let lics = self.local_interrupt_controllers.lock();
        if lics.is_empty() {
            return Err(GlobalInterruptControllerError::NoLocalInterruptControllers);
        }
        lics.iter()
            .filter(|lic| lic.available_local_sources_count() > 0)
            .max_by_key(|lic| lic.available_local_sources_count())
            .cloned()
            .ok_or(GlobalInterruptControllerError::OutOfLocalSources)
    }

    fn find_routing(
        &self,
        source: GlobalInterruptSource,
    ) -> Option<(Arc<dyn LocalInterruptController>, LocalInterruptSource)> {
        let lock = self.bindings.lock();
        let target = lock.get(&source)?;
        self.local_interrupt_controllers
            .lock()
            .iter()
            .find(|lic| lic.cpu_id() == target.cpu_id)
            .map(|lic| (lic.clone(), target.local_source))
    }
}

impl GlobalInterruptController for IoApicDevice {
    fn add_local_interrupt_controller(
        &self,
        local_interrupt_controller: Arc<dyn LocalInterruptController>,
    ) {
        self.local_interrupt_controllers
            .lock()
            .push(local_interrupt_controller);
    }

    fn allocate_target(
        &self,
    ) -> Result<(LocalInterruptTarget, Arc<InterruptSlot>), GlobalInterruptControllerError> {
        let local_interrupt_controller = self.select_local_interrupt_controller()?;
        let (local_source, slot) = local_interrupt_controller
            .allocate_local_source()
            .map_err(|err| GlobalInterruptControllerError::LocalInterruptControllerError(err))?;
        Ok((
            LocalInterruptTarget {
                cpu_id: local_interrupt_controller.cpu_id(),
                local_source,
            },
            slot,
        ))
    }

    fn route(
        &self,
        source: GlobalInterruptSource,
        target: LocalInterruptTarget,
    ) -> Result<(), GlobalInterruptControllerError> {
        let mut bindings = self.bindings.lock();
        let mut device = self.device.lock();

        if let Some(routed_target) = bindings.get(&source) {
            return Err(GlobalInterruptControllerError::AlreadyRouted(
                *routed_target,
            ));
        };
        if !device.has_gsi(source.0) {
            return Err(GlobalInterruptControllerError::InvalidGIS(source));
        }

        if !device.can_mask(source.0) {
            return Err(GlobalInterruptControllerError::NonMaskableInterrupt(source));
        }

        let flags = device.flags_for_gsi(source.0);
        device.set_irq(
            source.0,
            target.local_source.0 as u8,
            target.cpu_id.0 as u8,
            flags,
        );

        bindings.insert(source, target);
        Ok(())
    }

    fn unroute(&self, source: GlobalInterruptSource) -> Result<(), GlobalInterruptControllerError> {
        let mut bindings = self.bindings.lock();
        let mut device = self.device.lock();
        if !device.has_gsi(source.0) { return Err(GlobalInterruptControllerError::InvalidGIS(source)) }
        let (lic, local_source) = self.find_routing(source).ok_or(GlobalInterruptControllerError::NotRouted(source))?;
        lic.free_local_source(local_source).map_err(|e| GlobalInterruptControllerError::LocalInterruptControllerError(e))?;
        device.clear_irq(source.0);
        bindings.remove(&source);
        Ok(())
    }

    fn interrupt_target(&self, source: GlobalInterruptSource) -> Option<LocalInterruptTarget> {
        self.bindings.lock().get(&source).copied()
    }

    fn mask(&self, source: GlobalInterruptSource) -> Result<(), GlobalInterruptControllerError> {
        let mut device = self.device.lock();
        if !device.has_gsi(source.0) {
            return Err(GlobalInterruptControllerError::InvalidGIS(source));
        }
        if !device.can_mask(source.0) {
            return Err(GlobalInterruptControllerError::NonMaskableInterrupt(source))
        }
        device.set_masked(source.0, true);
        Ok(())
    }

    fn unmask(&self, source: GlobalInterruptSource) -> Result<(), GlobalInterruptControllerError> {
        let mut device = self.device.lock();
        if !device.has_gsi(source.0) {
            return Err(GlobalInterruptControllerError::InvalidGIS(source));
        }
        if !device.can_mask(source.0) {
            return Err(GlobalInterruptControllerError::NonMaskableInterrupt(source))
        }
        device.set_masked(source.0, false);
        Ok(())
    }

    fn routed_lines(&self) -> Vec<GlobalInterruptSource> {
        self.bindings
            .lock()
            .keys()
            .copied()
            .collect::<Vec<GlobalInterruptSource>>()
    }

    fn interrupt_count(&self, source: GlobalInterruptSource) -> Option<usize> {
        debug_assert!(
            self.device.lock().has_gsi(source.0),
            "GlobalInterruptController interrupt_count() was called with an invalid GIS!"
        );
        let (lic, local_source) = self.find_routing(source)?;
        Some(lic.slot(local_source)?.count())
    }

    fn total_interrupt_count(&self) -> usize {
        let guard = self.local_interrupt_controllers.lock();
        let mut sum = 0;
        for lic in &*guard {
            sum += lic.interrupts_count();
        }
        sum
    }
}

pub fn init_global(info: Apic) {
    disable_pic();
    let lapic_ptr =
        map_mmio_ptr(info.local_apic_address as usize, LAPIC_MMIO_SIZE).expect("MMIO map failed");
    LAPIC_PTR.store(lapic_ptr, Ordering::Release);
    ISO.call_once(|| info.interrupt_source_overrides.clone());
    let io_apic_dev = IoApicDevice {
        device: Mutex::new(ChainedIoApics::new(info)),
        local_interrupt_controllers: Mutex::new(Vec::new()),
        bindings: Mutex::new(BTreeMap::new()),
    };
    DEVICE_REGISTRY
        .write()
        .register(Arc::new(io_apic_dev) as Arc<dyn GlobalInterruptController>);
}

const DURATION_PER_TIMER_INTERRUPT: KernelDuration = KernelDuration::from_nanos(10_000_000);

pub fn init_lapic() -> LocalApic {
    let lapic_ptr = LAPIC_PTR.load(Ordering::Acquire);
    let mut lapic = LocalApicBuilder::new()
        .set_xapic_base(lapic_ptr as u64)
        .spurious_vector(SPURIOUS_VECTOR as usize)
        .error_vector(CONTROLLER_ERROR_VECTOR as usize)
        .timer_vector(TIMER_IRQ_VECTOR as usize)
        .timer_mode(TimerMode::OneShot)
        .timer_divide(TimerDivide::Div16)
        .timer_initial(0)
        .build()
        .expect("Local APIC build failed");
    unsafe {
        lapic.enable();
    }
    init_lapic_timer(&mut lapic);
    lapic
}

const VECTOR_COUNT: usize = 256;

pub struct SendLocalApic(LocalApic);

unsafe impl Send for SendLocalApic {}

impl SendLocalApic {
    pub fn get(&mut self, expected_cpu: CpuId) -> &mut LocalApic {
        debug_assert_eq!(
            current_cpu().logical_id,
            expected_cpu,
            "LAPIC touched from wrong core"
        );
        &mut self.0
    }
}

pub struct LocalApicDevice {
    lapic: InterruptSafeMutex<SendLocalApic>,
    slots: InterruptSafeMutex<[InterruptSourceState; VECTOR_COUNT]>,
    cpu_id: CpuId,
    interrupt_count: AtomicUsize,
    spurious_count: AtomicUsize,
}

impl LocalApicDevice {
    pub fn new(lapic: LocalApic, cpu_id: CpuId) -> Self {
        let dev = Self {
            lapic: InterruptSafeMutex::new(SendLocalApic(lapic)),
            slots: InterruptSafeMutex::new([const { InterruptSourceState::Free }; VECTOR_COUNT]),
            cpu_id,
            interrupt_count: AtomicUsize::new(0),
            spurious_count: AtomicUsize::new(0),
        };
        for i in 0..IRQ_BASE {
            dev.reserve(LocalInterruptSource(i as u32)).unwrap();
        }
        for reserved_vector in [TIMER_IRQ_VECTOR, SPURIOUS_VECTOR, CONTROLLER_ERROR_VECTOR] {
            dev.reserve(LocalInterruptSource(reserved_vector as u32)).unwrap();
        }
        dev
    }

    pub fn get_lapic_mutex(&self) -> &InterruptSafeMutex<SendLocalApic> {
        &self.lapic
    }
}

impl LocalInterruptController for LocalApicDevice {
    fn reserve(&self, source: LocalInterruptSource) -> Result<Arc<InterruptSlot>, LocalInterruptControllerError> {
        let slot = Arc::new(InterruptSlot::default());
        let mut slots = self.slots.lock();
        let entry = slots
            .get_mut(source.0 as usize)
            .ok_or(LocalInterruptControllerError::InvalidLIS(source))?;
        match entry {
            InterruptSourceState::Free => {
                *entry = InterruptSourceState::Reserved(Arc::clone(&slot));
                Ok(slot)
            }
            _ => Err(LocalInterruptControllerError::AlreadyAllocated(source)),
        }
    }

    fn allocate_local_source(
        &self,
    ) -> Result<(LocalInterruptSource, Arc<InterruptSlot>), LocalInterruptControllerError> {
        let slot = Arc::new(InterruptSlot::default());
        let mut slots = self.slots.lock();
        let index = slots.iter()
            .position(|s| matches!(s, InterruptSourceState::Free))
            .ok_or(LocalInterruptControllerError::OutOfSources)?;
        slots[index] = InterruptSourceState::Allocated(Arc::clone(&slot));
        Ok((LocalInterruptSource(index as u32), slot))
    }

    fn free_local_source(&self, source: LocalInterruptSource) -> Result<(), LocalInterruptControllerError> {
        let mut slots = self.slots.lock();
        let entry = slots
            .get_mut(source.0 as usize)
            .ok_or(LocalInterruptControllerError::InvalidLIS(source))?;
        match entry {
            InterruptSourceState::Allocated(_) => {
                *entry = InterruptSourceState::Free;
                Ok(())
            },
            InterruptSourceState::Reserved(_) => Err(LocalInterruptControllerError::SourceReserved(source)),
            InterruptSourceState::Free => Err(LocalInterruptControllerError::SourceNotAllocated)
        }
    }

    fn available_local_sources_count(&self) -> usize {
        self.slots.lock().iter().filter(|slot_state| matches!(slot_state, InterruptSourceState::Free)).count()
    }

    fn enter_interrupt(
        &self,
        source: LocalInterruptSource,
    ) -> Result<(), LocalInterruptControllerError> {
        self.interrupt_count.fetch_add(1, Ordering::Relaxed);

        let slot = self.slots.lock()
            .get(source.0 as usize)
            .and_then(|s| s.slot().cloned());

        let result = match slot {
            Some(slot) => { slot.dispatch(); Ok(()) }
            None => Err(LocalInterruptControllerError::SourceNotAllocated),
        };

        unsafe { self.lapic.lock().get(self.cpu_id).end_of_interrupt() };
        result
    }

    fn enter_spurious_interrupt(&self) {
        self.spurious_count.fetch_add(1, Ordering::Relaxed);
    }

    fn spurious_interrupts_count(&self) -> usize {
        self.spurious_count.load(Ordering::Relaxed)
    }

    fn can_mask(&self, _source: LocalInterruptSource) -> bool {
        false
    }

    fn mask(&self, _source: LocalInterruptSource) -> Result<(), LocalInterruptControllerError> {
        Err(LocalInterruptControllerError::MaskingNotSupported)
    }

    fn unmask(&self, _source: LocalInterruptSource) -> Result<(), LocalInterruptControllerError> {
        Err(LocalInterruptControllerError::MaskingNotSupported)
    }

    fn interrupts_count(&self) -> usize {
        self.interrupt_count.load(Ordering::Relaxed)
    }

    fn state(&self, source: LocalInterruptSource) -> InterruptSourceState {
        self.slots.lock().get(source.0 as usize).cloned().unwrap_or_default()
    }

    fn slot(&self, source: LocalInterruptSource) -> Option<Arc<InterruptSlot>> {
        self.state(source).slot().cloned()
    }

    fn cpu_id(&self) -> CpuId {
        self.cpu_id
    }

    fn interrupt_destination_id(&self) -> usize {
        unsafe { self.lapic.lock().get(self.cpu_id).id() as usize }
    }
}

fn init_lapic_timer(lapic: &mut LocalApic) {
    const CALIBRATION_ITERATION_COUNT: u32 = 5;
    let mut ticks_sum = 0;
    unsafe {
        lapic.enable_timer();
    }
    for _ in 0..CALIBRATION_ITERATION_COUNT {
        unsafe {
            lapic.set_timer_initial(u32::MAX);
        }
        crate::time::stall(DURATION_PER_TIMER_INTERRUPT);
        ticks_sum += u32::MAX - unsafe { lapic.timer_current() };
    }
    let tick_avg = ticks_sum / CALIBRATION_ITERATION_COUNT;
    unsafe {
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_initial(tick_avg);
    }
}

fn disable_pic() {
    let mut pic = unsafe { ChainedPics::new(0x20, 0x28) };
    unsafe {
        pic.initialize();
        pic.disable();
    }
}

fn flags_from(polarity: Polarity, trigger_mode: TriggerMode) -> IrqFlags {
    let mut flags = IrqFlags::empty();
    if let Polarity::ActiveLow = polarity {
        flags |= IrqFlags::LOW_ACTIVE
    }
    if let TriggerMode::Level = trigger_mode {
        flags |= IrqFlags::LEVEL_TRIGGERED
    }
    flags
}

static ISO: Once<Vec<InterruptSourceOverride>> = Once::new();

pub fn isa_irq_to_gsi(isa_irq: u8) -> GlobalInterruptSource {
    ISO.get().expect("No ISO Vec registered").iter().find(|iso| iso.isa_source == isa_irq).map(|iso| GlobalInterruptSource(iso.global_system_interrupt)).unwrap_or(GlobalInterruptSource(isa_irq as u32))
}
