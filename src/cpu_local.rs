use x2apic::lapic::LocalApic;
use x86_64::{VirtAddr, registers::model_specific::GsBase};

#[repr(C)]
pub struct CpuLocal {
    pub self_ptr: *mut CpuLocal,
    pub lapic: LocalApic,
    //pub current_task: *mut Task,
}

pub fn init(lapic: LocalApic) {
    let local = CpuLocal { 0 as *mut _, lapic };
    data.self_ptr = data as *mut _;
    let addr = VirtAddr::new(data as *mut _ as u64);
    GsBase::write(addr);
}

#[inline(always)]
pub fn current_cpu() -> &'static mut CpuLocal {
    let ptr: *mut CpuLocal;
    unsafe {
        core::arch::asm!(
            "mov {}, gs:0",
            out(reg) ptr,
            options(nostack, preserves_flags, readonly)
        );
        &mut *ptr
    }
}

