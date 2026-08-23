use core::sync::atomic::{AtomicUsize, Ordering};
use acpi::platform::interrupt::{Apic, InterruptSourceOverride, IoApic as IoApicInfo};
use acpi::sdt::madt::{Polarity, TriggerMode};
use alloc::sync::Arc;
use alloc::vec::Vec;
use pic8259::ChainedPics;
use spin::Mutex;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::structures::paging::PageSize;

use crate::dev::registry::DEVICE_REGISTRY;
use crate::dev::traits::GlobalIrqController;
use crate::memory::FrameSize;
use crate::{memory::map_mmio_ptr, time::KernelDuration};

use super::{GlobalInterruptController, LocalInterruptController};

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

impl ChainedIoApics {
    fn new(info: Apic) -> Self {
        let mut io_apics = Vec::new();
        for io_apic_info in info.io_apics {
            let mmio_addr = map_mmio_ptr(io_apic_info.address as usize, FrameSize::SIZE as usize).expect("MMIO mapping failed") as u64;
            let mut io_apic = unsafe { IoApic::new(mmio_addr) };
            let max_entry = unsafe { io_apic.max_table_entry() };
            for irq in 0..=max_entry {
                let mut entry = RedirectionTableEntry::default();
                entry.set_mode(IrqMode::Fixed);
                entry.set_flags(IrqFlags::MASKED);
                unsafe { io_apic.set_table_entry(irq, entry); }
            }
            io_apics.push(IoApicEntry { info: io_apic_info, io_apic, max_entry });
        }
        let mut chained_io_apics = Self { io_apics, iso: info.interrupt_source_overrides };
        for nmi_source in info.nmi_sources {
            let io_apic_entry = chained_io_apics.get_ioapic_entry(nmi_source.global_system_interrupt).expect("IOAPIC with corrponding GSI number doesn't exist");
            let irq = (nmi_source.global_system_interrupt - io_apic_entry.info.global_system_interrupt_base) as u8;
            let mut entry = unsafe { io_apic_entry.io_apic.table_entry(irq) };
            entry.set_mode(IrqMode::NonMaskable);
            let mut flags = entry.flags();
            flags.remove(IrqFlags::MASKED);
            if let Polarity::ActiveLow = nmi_source.polarity { flags |= IrqFlags::LOW_ACTIVE; }
            if let TriggerMode::Level = nmi_source.trigger_mode { flags |= IrqFlags::LEVEL_TRIGGERED; }
            entry.set_flags(flags);
            unsafe { io_apic_entry.io_apic.set_table_entry(irq, entry); }
        }
        chained_io_apics
    }

    fn get_ioapic_entry(&mut self, gsi: u32) -> Option<&mut IoApicEntry> {
        self.io_apics.iter_mut().find(|e| {
            let base = e.info.global_system_interrupt_base;
            gsi >= base && gsi <= base + e.max_entry as u32
        })
    }

    fn resolve_isa(&self, isa: u8) -> (u32, IrqFlags) {
        match self.iso.iter().find(|o| o.isa_source == isa) {
            Some(o) => (o.global_system_interrupt, flags_from(o.polarity, o.trigger_mode)),
            None => (isa as u32, IrqFlags::empty()), // edge, active-high
        }
    }

    fn set_irq(&mut self, gsi: u32, vector: u8, dest: u8, flags: IrqFlags) {
        let io_apic_entry = self.get_ioapic_entry(gsi).expect("IOAPIC with corrponding GSI number doesn't exist");
        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_vector(vector);
        entry.set_dest(dest);
        entry.set_flags(flags | IrqFlags::MASKED);
        unsafe { io_apic_entry.io_apic.set_table_entry((gsi - io_apic_entry.info.global_system_interrupt_base) as u8, entry); }
    }

    fn set_masked(&mut self, gsi: u32, masked: bool) {
        let io_apic_entry = self.get_ioapic_entry(gsi).expect("IOAPIC with corrponding GSI number doesn't exist");
        let mut entry = unsafe { io_apic_entry.io_apic.table_entry((gsi - io_apic_entry.info.global_system_interrupt_base) as u8) };
        let mut flags = entry.flags();
        flags.set(IrqFlags::MASKED, masked);
        entry.set_flags(flags);
        unsafe { io_apic_entry.io_apic.set_table_entry((gsi - io_apic_entry.info.global_system_interrupt_base) as u8, entry); }
    }
}

struct IoApicDevice {
    device: Mutex<ChainedIoApics>,
    local_interrupt_controllers: Vec<Arc<dyn LocalInterruptController>>,
}

impl GlobalInterruptController for IoApicDevice {
    fn add_local_interrupt_controller(&self, local_interrupt_controller: Arc<dyn super::LocalInterruptController>) {
    }

    fn register_event(&self, source: super::GlobalInterruptSource, event: alloc::sync::Weak<super::IrqEvent>) -> Result<(), super::GICError> {
    }

    fn gis_count(&self, source: super::GlobalInterruptSource) -> u64 {
    }

    fn total_gis_count(&self) -> u64 {
    }

    fn mask(&self, source: super::GlobalInterruptSource) {
        self.device.lock().set_masked(source.0, true);
    }

    fn unmask(&self, source: super::GlobalInterruptSource) {
        self.device.lock().set_masked(source.0, true);
    }
}

pub fn init_global(info: Apic) {
    disable_pic();
    let lapic_ptr = map_mmio_ptr(info.local_apic_address as usize, LAPIC_MMIO_SIZE).expect("MMIO map failed");
    LAPIC_PTR.store(lapic_ptr, Ordering::Release);
    let io_apic_dev = IoApicDevice(Mutex::new(ChainedIoApics::new(info)));
    DEVICE_REGISTRY.write().register(Arc::new(io_apic_dev) as Arc<dyn GlobalIrqController>);
}

pub fn init_lapic() -> LocalApic {
    let lapic_ptr = LAPIC_PTR.load(Ordering::Acquire);
    let mut lapic = LocalApicBuilder::new()
        .set_xapic_base(lapic_ptr as u64)
        .spurious_vector(super::SUPRIOUS_VECTOR as usize)
        .error_vector(super::CONTROLLER_ERROR_VECTOR as usize)
        .timer_vector(super::TIMER_VECOTR as usize)
        .timer_mode(TimerMode::OneShot)
        .timer_divide(TimerDivide::Div16)
        .timer_initial(0)
        .build()
        .expect("Local APIC build failed");
    unsafe {
        lapic.enable();
    }
    lapic
}

pub fn init_lapic_timer(lapic: &mut LocalApic, nanos_per_int: u64) {
    const CALIBRATION_ITERATION_COUNT: u32 = 5;
    let mut ticks_sum = 0;
    unsafe {
        lapic.enable_timer();
    }
    for _ in 0..CALIBRATION_ITERATION_COUNT {
        unsafe {
            lapic.set_timer_initial(u32::MAX);
        }
        crate::time::stall(KernelDuration::from_nanos(nanos_per_int));
        ticks_sum += u32::MAX - unsafe { lapic.timer_current() };
    }
    let tick_avrg = ticks_sum / CALIBRATION_ITERATION_COUNT;
    unsafe {
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_initial(tick_avrg);
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
    if let Polarity::ActiveLow = polarity { flags |= IrqFlags::LOW_ACTIVE }
    if let TriggerMode::Level = trigger_mode { flags |= IrqFlags::LEVEL_TRIGGERED }
    flags
}

