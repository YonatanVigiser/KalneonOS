use acpi::platform::ProcessorInfo;
use spin::Once;
use core::sync::atomic::Ordering;

use boot::BootInfo;

mod boot;
pub mod cpu;
pub mod gdt;
mod idt;
mod smp;

pub static BOOT_INFO: Once<BootInfo> = Once::new();

pub fn init_boot(boot_magic: u32, boot_info_ptr: u32) -> &'static BootInfo {
    BOOT_INFO.call_once(|| boot::load_boot_info(boot_magic, boot_info_ptr))
}

pub fn init_cpu(uid: u32, logical_id: usize) {
    unsafe { gdt::load() };
    unsafe { idt::load() };
    let lapic = crate::interrupt::init_local();
    cpu::init(uid, logical_id, lapic);
}

pub fn init_smp(processor_info: &ProcessorInfo) {
    log::info!("Starting SMP...");
    unsafe { smp::start(&mut cpu::current_cpu().lapic, processor_info) };
}

pub unsafe fn halt_smp() {
    if cores_count() > 1 {
        let lapic = &mut cpu::current_cpu().lapic;
        unsafe { smp::halt(lapic) }
    }
}

pub fn cores_count() -> usize {
    smp::ACTIVE_PROCESSORS_COUNTER.load(Ordering::Relaxed)
}
