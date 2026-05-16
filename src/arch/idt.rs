use lazy_static::lazy_static;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::set_general_handler;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::interrupt::handlers::*;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        set_general_handler!(&mut idt, general_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.breakpoint.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_maskable_handler);
        idt[crate::interrupt::TIMER_VECOTR].set_handler_fn(timer_irq_handler);
        idt
    };
}

pub unsafe fn load() {
    without_interrupts(|| IDT.load() );
}
