use acpi::platform::ProcessorInfo;
use spin::Once;
use core::sync::atomic::{AtomicBool, Ordering};

use boot::BootInfo;

use crate::interrupt::LocalInterruptController;

use self::cpu::CpuId;

mod boot;
pub mod cpu;
pub mod gdt;
mod idt;
mod smp;

pub static BOOT_INFO: Once<BootInfo> = Once::new();

pub fn init_boot(boot_magic: u32, boot_info_ptr: u32) -> &'static BootInfo {
    BOOT_INFO.call_once(|| boot::load_boot_info(boot_magic, boot_info_ptr))
}

pub fn init_cpu(uid: u32, logical_id: CpuId) {
    unsafe { gdt::load() };
    unsafe { idt::load() };
    cpu::init(uid, logical_id);
}

pub fn init_smp(processor_info: &ProcessorInfo) {
    log::info!("Starting SMP...");
    let mut lapic = crate::interrupt::init_local();
    unsafe { smp::start(&mut lapic, processor_info) };
    crate::interrupt::register_local(lapic);
}

pub unsafe fn halt_smp() {
    static HALTING_SMP: AtomicBool = AtomicBool::new(false);
    if cores_count() > 1 && !HALTING_SMP.swap(true, Ordering::Release) {
        if let Some(lapic_dev) = cpu::current_cpu().lapic.as_ref() {
            let mut guard = unsafe { lapic_dev.get_lapic_mutex().make_guard_unchecked() };
            let lapic = guard.get(lapic_dev.cpu_id());
            unsafe { smp::halt(lapic) }
        }
    }
}

pub fn cores_count() -> usize {
    smp::ACTIVE_PROCESSORS_COUNTER.load(Ordering::Relaxed)
}
