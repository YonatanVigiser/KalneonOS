use lazy_static::lazy_static;
use x86_64::set_general_handler;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        set_general_handler!(&mut idt, general_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

fn general_handler(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
    panic!("Unhandled interrupt: {}\nStack frame:\n{:?}\nError Code: {:?}", index, stack_frame, error_code);
}

