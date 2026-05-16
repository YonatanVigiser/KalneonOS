use x2apic::lapic::LocalApic;
use x86_64::{VirtAddr, registers::model_specific::GsBase};

use alloc::boxed::Box;

#[repr(C)]
pub struct CpuLocal {
    self_ptr: *mut CpuLocal,
    pub logical_id: usize,
    pub processor_uid: u32,
    interrupts_depth: u16,
    pub lapic: LocalApic,
    pub kernel_stack_top: u64,
}

impl CpuLocal {
    #[inline]
    pub fn enter_interrupt(&mut self) { self.interrupts_depth += 1; }

    #[inline]
    pub fn leave_interrupt(&mut self) { self.interrupts_depth -= 1; }

    #[inline]
    pub fn interrupt_depth(&self) -> u16 { self.interrupts_depth }
}

pub(super) fn init(uid: u32, logical_id: usize, lapic: LocalApic) {
    let local = Box::leak(Box::new(CpuLocal {
        self_ptr: core::ptr::null_mut(),
        logical_id,
        processor_uid: uid,
        lapic,
        interrupts_depth: 0,
        kernel_stack_top: 0,
    }));
    local.self_ptr = local as *mut CpuLocal;
    let addr = VirtAddr::new(local.self_ptr as u64);
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
