use x2apic::lapic::LocalApic;
use x86_64::{VirtAddr, registers::model_specific::GsBase};

use alloc::{boxed::Box, collections::VecDeque};
use crate::task::TaskId;

#[repr(C)]
pub struct CpuLocal {
    pub self_ptr: *mut CpuLocal,
    pub cpu_id: u32,
    pub interrupts_depth: u16,
    pub lapic: LocalApic,
    pub tasks: VecDeque<TaskId>,
    //pub current_task: *mut Task,
}

pub fn init(cpu_id: u32, lapic: LocalApic) {
    let local = Box::leak(Box::new(CpuLocal { self_ptr: core::ptr::null_mut(), cpu_id, lapic, interrupts_depth: 0, tasks: VecDeque::new() }));
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

