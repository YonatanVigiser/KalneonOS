use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

use crate::arch::cpu::current_cpu;

use super::{LocalInterruptController, LocalInterruptSource};

macro_rules! _interrupt_handler {
    ($handler:ident) => {{
        #[naked]
        unsafe extern "x86-interrupt" fn wrapper() {
            naked_asm!(
                "swapgs",
                "push rax", "push rbx", "push rcx", "push rdx",
                "push rsi", "push rdi", "push rbp",
                "push r8",  "push r9",  "push r10", "push r11",
                "push r12", "push r13", "push r14", "push r15",
                "rdfsbase rax", "push rax", "rdgsbase rax", "push rax",
                "call {f}",
                "pop rax", "wrfsbase rax", "pop rax", "wrgsbase rax",
                "pop r15", "pop r14", "pop r13", "pop r12",
                "pop r11", "pop r10", "pop r9",  "pop r8",
                "pop rbp",  "pop rdi", "pop rsi", "pop rdx",
                "pop rcx",  "pop rbx", "pop rax",
                "swapgs",
                "iretq",
                f = sym $handler,
            );
        }
        wrapper
    }};
}

pub fn general_handler(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
    if let Some(error_code) = error_code {
        panic!(
            "Unhandled exception: {}\nStack frame:\n{:?}\nError Code: {:?}",
            index, stack_frame, error_code
        );
    } else {
        let lic = current_cpu().lapic.as_mut().expect("No local interrupt controller");
        lic.enter_interrupt(LocalInterruptSource(index as u32)).expect("local interrupt controller handling failed");
    }
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    panic!(
        "EXCEPTION: PAGE FAULT (core: {})\nAccessed Address: {:?}\nError Code: {:?}\n{:#?}",
        crate::arch::cpu::current_cpu().logical_id,
        Cr2::read(),
        error_code,
        stack_frame
    );
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "Double Fault! Error Code: {}. Stack Frame:\n{:#?}",
        error_code, stack_frame
    )
}

pub extern "x86-interrupt" fn non_maskable_handler(_stack_frame: InterruptStackFrame) {
    crate::halt_loop()
}

pub extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {}
