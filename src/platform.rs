pub mod acpi;
pub mod apic;
pub mod boot;
pub mod cpu;
mod gdt;
mod idt;
pub mod paging;
pub mod smp;

pub fn init_boot() {
}

pub fn init_cpu(platform_uid: usize, logical_id: usize) {
    unsafe { gdt::load() };
    unsafe { idt::load() };
    let lapic = apic::init_lapic();
    cpu::init(platform_uid, logical_id, lapic);
}

pub fn init_smp() -> ! {
}
