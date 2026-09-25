use core::cell::{Cell, OnceCell};
use core::fmt::Display;

use alloc::sync::Arc;
use x86_64::{VirtAddr, registers::model_specific::GsBase};

use crate::interrupt::apic::LocalApicDevice;
use crate::task::TaskId;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuId(pub usize);

impl Display for CpuId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CPUID-{}", self.0)
    }
}

#[repr(C)]
pub struct CpuLocal {
    self_ptr: *const CpuLocal,
    pub logical_id: CpuId,
    pub processor_uid: u32,
    interrupts_depth: Cell<u16>,
    pub lapic: OnceCell<Arc<LocalApicDevice>>,
    pub kernel_stack_top: Cell<u64>,
    pub current_task_id: Cell<Option<TaskId>>,
}

impl CpuLocal {
    pub const fn new(processor_uid: u32, logical_id: CpuId) -> Self {
        Self {
            self_ptr: core::ptr::null_mut(),
            logical_id,
            processor_uid,
            lapic: OnceCell::new(),
            interrupts_depth: Cell::new(0),
            kernel_stack_top: Cell::new(0),
            current_task_id: Cell::new(None),
        }
    }
}

impl CpuLocal {
    #[inline]
    pub fn enter_interrupt(&self) { self.interrupts_depth.update(|v| v + 1); }

    #[inline]
    pub fn leave_interrupt(&self) { self.interrupts_depth.update(|v| v - 1); }

    #[inline]
    pub fn interrupt_depth(&self) -> u16 { self.interrupts_depth.get() }
}

pub static mut BSP_CPU_LOCAL: CpuLocal = CpuLocal::new(0, CpuId(0));

pub(super) unsafe fn install(local: *mut CpuLocal) {
    unsafe { (*local).self_ptr = local; }
    GsBase::write(VirtAddr::new(local as u64));
}

#[inline(always)]
pub fn current_cpu() -> &'static CpuLocal {
    let ptr: *const CpuLocal;
    unsafe {
        core::arch::asm!(
            "mov {}, gs:0",
            out(reg) ptr,
            options(nostack, preserves_flags, readonly)
        );
        &*ptr
    }
}
