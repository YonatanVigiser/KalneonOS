use acpi::platform::InterruptModel;
use acpi::platform::AcpiPlatform;
use spin::Once;
use core::sync::atomic::Ordering;

use boot::BootInfo;

mod acpi;
mod apic;
mod boot;
pub mod cpu;
pub mod gdt;
mod idt;
mod paging;
mod smp;

static BOOT_INFO: Once<BootInfo> = Once::new();
static ACPI: Once<AcpiPlatform> = Once::new();

pub fn init_boot(boot_magic: u32, boot_info_ptr: u32) {
    let boot_info = BOOT_INFO.call_once(|| boot::load_boot_info(boot_magic, boot_info_ptr));
    memory::init(&boot_info.mmap, || {
        let mut guard = crate::drivers::display::vga::VGA.lock();
        let vga = guard.as_mut().expect("VGA not initialized before paging");
        let ptr = memory::map_mmio_ptr(vga.get_ptr() as usize, vga.get_buffer_size())
        .expect("VGA MMIO remap failed") as *mut u16;
        vga.update_ptr(ptr);
    });
    let acpi = ACPI.call_once(|| acpi::platform_info(boot_info.rsdt_addr, boot_info.rsdt_revision));
    match acpi.interrupts_model {
        InterruptModel::Apic(apic) => {
            apic::set_lapic_addr(apic.local_apic_address as usize)
        },
        InterruptModel::Unknown => panic!("Unsupported interrupts mode!"),
    };
    init_cpu(acpi.processor_info.boot_processor.processor_uid, 0);
}

pub fn init_cpu(uid: u32, logical_id: usize) {
    unsafe { gdt::load() };
    unsafe { idt::load() };
    let lapic = apic::init_lapic();
    cpu::init(uid, logical_id, lapic);
}

pub fn init_smp() -> usize {
    log::info!("Starting SMP...");
    unsafe { smp::start(&mut cpu::current_cpu().lapic, ACPI.get().unwrap().processor_info()) };
}

pub fn cores_count() -> usize {
    smp::ACTIVE_PROCESSORS_COUNTER.load(Ordering::Relaxed)
}
