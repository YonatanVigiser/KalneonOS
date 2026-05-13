macro_rules! interrupt_handler {
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

unsafe extern "C" fn timer_irq_handler() {
    unsafe {
        crate::platform::cpu::current_cpu().lapic.end_of_interrupt();
    }
}
