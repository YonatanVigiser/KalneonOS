use core::fmt::Display;

use alloc::sync::Arc;
use x86_64::{VirtAddr, registers::model_specific::GsBase};

use alloc::boxed::Box;

use crate::interrupt::apic::LocalApicDevice;
use crate::task::TaskId;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuId(pub usize);

impl Display for CpuId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Logical CPU ID: {}", self.0)
    }
}

#[repr(C)]
pub struct CpuLocal {
    self_ptr: *mut CpuLocal,
    pub logical_id: CpuId,
    pub processor_uid: u32,
    interrupts_depth: u16,
    pub lapic: Option<Arc<LocalApicDevice>>,
    pub kernel_stack_top: u64,
    pub current_task_id: Option<TaskId>,
}

impl CpuLocal {
    #[inline]
    pub fn enter_interrupt(&mut self) { self.interrupts_depth += 1; }

    #[inline]
    pub fn leave_interrupt(&mut self) { self.interrupts_depth -= 1; }

    #[inline]
    pub fn interrupt_depth(&self) -> u16 { self.interrupts_depth }
}

pub(super) fn init(uid: u32, logical_id: CpuId) {
    let local = Box::leak(Box::new(CpuLocal {
        self_ptr: core::ptr::null_mut(),
        logical_id,
        processor_uid: uid,
        lapic: None,
        interrupts_depth: 0,
        kernel_stack_top: 0,
        current_task_id: None,
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
