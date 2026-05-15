use ::acpi::HpetInfo;
use acpi_crate::platform::InterruptModel;
use acpi_crate::platform::AcpiPlatform;
use spin::Once;
use core::sync::atomic::Ordering;

use boot::BootInfo;

extern crate acpi as acpi_crate;

mod acpi;
mod apic;
mod boot;
pub mod cpu;
pub mod gdt;
mod idt;
pub mod paging;
mod smp;
mod hpet;

static BOOT_INFO: Once<BootInfo> = Once::new();
static ACPI: Once<AcpiPlatform<acpi::AcpiRuntimeHandler>> = Once::new();

pub fn init_boot(boot_magic: u32, boot_info_ptr: u32) {
    let boot_info = BOOT_INFO.call_once(|| boot::load_boot_info(boot_magic, boot_info_ptr));
    crate::memory::init(&boot_info.mmap, || {
        let mut guard = crate::drivers::display::vga::VGA.lock();
        let vga = guard.as_mut().expect("VGA not initialized before paging");
        let ptr = crate::memory::map_mmio_ptr(vga.get_ptr() as usize, vga.get_buffer_size())
        .expect("VGA MMIO remap failed") as *mut u16;
        vga.update_ptr(ptr);
    });
    let acpi = ACPI.call_once(|| acpi::platform_info(boot_info.rsdt_addr, boot_info.rsdt_revision));
    match acpi.interrupt_model {
        InterruptModel::Apic(ref apic) => {
            apic::set_lapic_addr(apic.local_apic_address as usize)
        },
        _ => panic!("Unsupported interrupts mode!"),
    };
    hpet::init_hpet(HpetInfo::new(&acpi.tables).expect("No HPET info in ACPI tables!"));
    init_cpu(acpi.processor_info.as_ref().expect("No processor info in MADT").boot_processor.processor_uid, 0);
}

pub fn init_cpu(uid: u32, logical_id: usize) {
    unsafe { gdt::load() };
    unsafe { idt::load() };
    let lapic = apic::init_lapic();
    cpu::init(uid, logical_id, lapic);
}

pub fn init_smp() {
    log::info!("Starting SMP...");
    let processor_info = &ACPI.get().as_ref().unwrap().processor_info.as_ref().expect("No processor info in MADT");
    unsafe { smp::start(&mut cpu::current_cpu().lapic, processor_info) };
}

pub unsafe fn halt_smp() {
    let lapic = &mut cpu::current_cpu().lapic;
    unsafe { smp::halt(lapic) }
}

pub fn cores_count() -> usize {
    smp::ACTIVE_PROCESSORS_COUNTER.load(Ordering::Relaxed)
}

pub fn uptime_nano() -> u64 {
    hpet::uptime_nano()
}
