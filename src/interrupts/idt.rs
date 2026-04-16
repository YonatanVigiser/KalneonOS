use lazy_static::lazy_static;
use x86_64::set_general_handler;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        set_general_handler!(&mut idt, general_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[super::apic::TIMER_VECOTR].set_handler_fn(timer_irq_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

fn general_handler(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
    panic!(
        "Unhandled interrupt: {}\nStack frame:\n{:?}\nError Code: {:?}",
        index, stack_frame, error_code
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    panic!(
        "EXCEPTION: PAGE FAULT\nAccessed Address: {:?}\nError COde: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "Double Fault! Error Code: {}. Stack Frame:\n{:#?}",
        error_code, stack_frame
    )
}

extern "x86-interrupt" fn timer_irq_handler(_stack_frame: InterruptStackFrame) {
    log::info!("Timer IRQ! HPET time: {:?}", crate::drivers::uptime_nano());
    unsafe { crate::cpu_local::current_cpu().lapic.end_of_interrupt(); }
}
