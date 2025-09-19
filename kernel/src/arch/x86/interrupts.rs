use super::pic;
use crate::debug_hex;
use super::drivers::pit;

#[repr(C)]
pub struct IntteruptStackFrame {
    gs: u32,
    fs: u32,
    es: u32,
    ds: u32,
    edi: u32,
    esi: u32,
    ebp: u32,
    esp: u32,
    ebx: u32,
    edx: u32,
    ecx: u32,
    eax: u32,
    int_num: u32,
    err_code: u32,
    eip: u32,
    cs: u32,
    eflags: u32,
    useresp: u32,
    ss: u32,
}

debug_hex!(IntteruptStackFrame,
  hex: [ss, useresp, eflags, cs, eip, err_code, int_num, eax, ecx, edx, ebx, esp, ebp, esi, edi, ds, es, fs, gs],
  normal: []
);

#[unsafe(no_mangle)]
pub extern "C" fn intterupts_handler(stack_frame: &mut IntteruptStackFrame) {
    match stack_frame.int_num {
        32 => irq0_handler(),
        39 => {
            if (pic::read_isr() & 0x0F) == 39 {
                pic::spurios_irq(true);
            } else {
                intterupt_panic(stack_frame);
            }
        }
        47 => {
            if (pic::read_isr() & 0xF0) == 47 {
                pic::spurios_irq(false);
            } else {
                intterupt_panic(stack_frame);
            }
        }
        _ => intterupt_panic(stack_frame),
    };
}

fn irq0_handler() {
    pit::hardware_interrupt();
    pic::send_eoi(32);
}

fn intterupt_panic(stack_frame: &mut IntteruptStackFrame) {
    panic!("{:?}", stack_frame);
}

pub struct Controller();
