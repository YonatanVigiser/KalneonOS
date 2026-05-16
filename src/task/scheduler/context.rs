use crate::arch::cpu::CpuLocal;

const KERNEL_STACK_TOP_OFFSET: usize = core::mem::offset_of!(CpuLocal, kernel_stack_top);
core::arch::global_asm!(
    "kernel_context_switch:",
    "push rbx",
    "push rbp",
    "push r12",
    "push r13",
    "push r14",
    "push r15",
    "mov gs:{0}, rsp",
    "mov rsp, rdi",
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop rbp",
    "pop rbx",
    "ret",
    const KERNEL_STACK_TOP_OFFSET
);

core::arch::global_asm!("");

unsafe extern "C" {
    fn kernel_context_switch(new_stack_top: usize);
    fn context_ret(context: CpuContext);
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CpuContext {
    pub fsbase: u64,
    pub gsbase: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

use x86_64::VirtAddr;
use x86_64::registers::{
    rflags::RFlags,
    segmentation::{FS, GS, Segment64},
};

impl CpuContext {
    pub fn new_user(_code_entry: VirtAddr, _stack_top: VirtAddr) -> Self {
        todo!("user mode selectors not yet defined in GDT")
    }

    pub fn new_kernel(code_entry: VirtAddr, stack_top: VirtAddr) -> Self {
        let starting_flags = RFlags::ID | RFlags::INTERRUPT_FLAG;
        Self {
            fsbase: FS::read_base().as_u64(),
            gsbase: GS::read_base().as_u64(),
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rdi: 0,
            rsi: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: code_entry.as_u64(),
            cs: crate::arch::gdt::kernel_code_selector().0 as u64,
            rflags: starting_flags.bits(),
            rsp: stack_top.as_u64(),
            ss: crate::arch::gdt::kernel_data_selector().0 as u64,
        }
    }
}
