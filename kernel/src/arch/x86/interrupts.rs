use super::pic;
use crate::debug_hex;

#[repr(C)]
pub struct InterruptStackFrame {
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
}

debug_hex!(InterruptStackFrame,
  hex: [eflags, cs, eip, err_code, int_num, eax, ecx, edx, ebx, esp, ebp, esi, edi, ds, es, fs, gs],
  normal: []
); 

static mut INTS_HANDLERS: [fn(&mut InterruptStackFrame); 256] = [interrupt_panic; 256];

#[unsafe(no_mangle)]
pub extern "C" fn interrupts_handler(stack_frame: &mut InterruptStackFrame) {
    write!(unsafe { crate::arch::x86::VIDEO_CONSOLE_REF.unwrap().as_mut() }, "{}", stack_frame.int_num);
    match stack_frame.int_num {
        0x27 => {
            if (pic::read_isr() & 0xFF) == 0x27 {
                pic::spurios_irq(true)
            } else {
                unsafe { INTS_HANDLERS[stack_frame.int_num as usize](stack_frame) }
            }
        }
        0x2F => {
            if ((pic::read_isr() & 0xFF00) >> 8) == 0x2F {
                pic::spurios_irq(false)
            } else {
                unsafe { INTS_HANDLERS[stack_frame.int_num as usize](stack_frame) }
            }
        }
        _ => unsafe { INTS_HANDLERS[stack_frame.int_num as usize](stack_frame) },
    };
}

fn interrupt_panic(stack_frame: &mut InterruptStackFrame) {
    panic!("Intterupt num: {}, is unimplemented! Stack frame:\n{:?}", stack_frame.int_num, stack_frame);
}

pub fn register_interrupt_handler(int_num: u8, handler: fn(&mut InterruptStackFrame)) {
    unsafe { INTS_HANDLERS[int_num as usize] = handler; }
}
