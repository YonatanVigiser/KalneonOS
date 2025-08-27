use super::pic;

#[unsafe(no_mangle)]
pub extern "C" fn intterupts_handler(int_num: u32, error_code: u32) {
    match int_num {
        39 => {
            if (pic::read_isr() & 0x0F) == 39 {
                pic::spurios_irq(true);
            } else {
                intterupt_panic(int_num, error_code);
            }
        }
        47 => {
            if (pic::read_isr() & 0xF0) == 47 {
                pic::spurios_irq(false);
            } else {
                intterupt_panic(int_num, error_code);
            }
        }
        _ => intterupt_panic(int_num, error_code),
    };
}

fn intterupt_panic(int_num: u32, error_code: u32) {
    panic!("Intterupt! Num: {int_num}, error_code: {error_code}");
}
