use core::sync::atomic::{AtomicUsize, Ordering};
use acpi::platform::interrupt::{Apic, IoApic as IoApicInfo};
use acpi::sdt::madt::{Polarity, TriggerMode};
use alloc::vec::Vec;
use pic8259::ChainedPics;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};
use x86_64::structures::paging::PageSize;

use crate::memory::FrameSize;
use crate::{memory::map_mmio_ptr, time::KernelDuration};

const LAPIC_MMIO_SIZE: usize = 0x1000;

pub const IRQ_BASE: u8 = 0x20;

pub static LAPIC_ADDR: AtomicUsize = AtomicUsize::new(0);

pub struct IoApicBase(u32);

pub struct ChainedIoApics {
    io_apics: Vec<(IoApicInfo, IoApic)>,
}

impl ChainedIoApics {
    pub fn new(io_apics_info: Vec<IoApicInfo>) -> Self {
        let mut io_apics = Vec::new();
        for io_apic_info in io_apics_info {
            let mmio_addr = map_mmio_ptr(io_apic_info.address as usize, FrameSize::SIZE as usize).expect("MMIO mapping failed") as u64;
            let io_apic = unsafe { IoApic::new(mmio_addr) };
            io_apics.push((io_apic_info, io_apic));
        }
        Self { io_apics }
    }

    pub fn get_ioapic(&mut self, gsi: u32) -> Option<(&IoApicInfo, &mut IoApic)> {
        for io_apic in &mut self.io_apics {
            let base = io_apic.0.global_system_interrupt_base;
            let end = base + unsafe { io_apic.1.max_table_entry() } as u32;
            if gsi >= base && gsi <= end {
                return Some((&io_apic.0, &mut io_apic.1));
            }
        }
        None
    }

    pub fn set_irq(&mut self, gsi: u32, vector: u8, dest: u8) {
        let io_apic = self.get_ioapic(gsi);
        let (_, io_apic) = io_apic.expect("IOAPIC with corrponding GSI number doesn't exist");
    }
}

pub fn init_global(info: Apic) {
    disable_pic();
    LAPIC_ADDR.store(info.local_apic_address as usize, Ordering::Relaxed);
    let mut io_apics = ChainedIoApics::new(info.io_apics);
    for iso in info.interrupt_source_overrides {
        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_vector(iso.isa_source + IRQ_BASE);
        let mut flags = IrqFlags::MASKED;
        if let Polarity::ActiveLow = iso.polarity {
            flags |= IrqFlags::LOW_ACTIVE;
        }
        if let TriggerMode::Level = iso.trigger_mode {
            flags |= IrqFlags::LEVEL_TRIGGERED;
        }
        entry.set_flags(flags);
        let (io_apic_info, io_apic) = io_apics.get_ioapic(iso.global_system_interrupt).expect("IOAPIC with corrponding GSI number doesn't exist");
        unsafe { io_apic.set_table_entry((iso.global_system_interrupt - io_apic_info.global_system_interrupt_base) as u8, entry); }
    }
    for nmi_source in info.nmi_sources {
        let (io_apic_info, io_apic) = io_apics.get_ioapic(nmi_source.global_system_interrupt).expect("IOAPIC with corrponding GSI number doesn't exist");
        let irq = (nmi_source.global_system_interrupt - io_apic_info.global_system_interrupt_base) as u8;
        let mut entry = unsafe { io_apic.table_entry(irq) };
        entry.set_mode(IrqMode::NonMaskable);
        let mut flags = entry.flags();
        flags.remove(IrqFlags::MASKED);
        if let Polarity::ActiveLow = nmi_source.polarity { flags |= IrqFlags::LOW_ACTIVE; }
        if let TriggerMode::Level = nmi_source.trigger_mode { flags |= IrqFlags::LEVEL_TRIGGERED; }
        entry.set_flags(flags);
        unsafe { io_apic.set_table_entry(irq, entry); }
    }
}

pub fn init_lapic() -> LocalApic {
    let lapic_ptr =
        map_mmio_ptr(LAPIC_ADDR.load(Ordering::Relaxed), LAPIC_MMIO_SIZE).expect("MMIO map failed");
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
