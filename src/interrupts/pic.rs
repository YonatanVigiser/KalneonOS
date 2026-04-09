use pic8259::ChainedPics;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

static SPURIOUS_IRQS_COUNT: AtomicUsize = AtomicUsize::new(0);

pub const PIC1_OFFSET: u8 = 0x20;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

pub static mut PICS: UnsafeCell<ChainedPics> = UnsafeCell::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

pub fn init() {
    unsafe { PICS.initialize(); } 
}

pub fn send_eoi(irq_index: u8) {
    unsafe { PICS.notify_end_of_interrupt(irq_index + PIC1_OFFSET); }
}

pub fn mask(irq_index: u8) {
    let masks = unsafe { PICS.read_masks() };
    if irq_index < PIC2_OFFSET {
        unsafe { PICS.write_masks(masks[0] | (1 << irq_index), masks[1]); }
    } else {
        unsafe { PICS.write_masks(masks[0], masks[1] | (1 << irq_index)); }
    }
}

pub fn unmask(irq_index: u8) {
    let masks = unsafe { PICS.read_masks() };
    if irq_index < PIC2_OFFSET {
        unsafe { PICS.write_masks(masks[0] & !(1 << irq_index), masks[1]); }
    } else {
        unsafe { PICS.write_masks(masks[0], masks[1] & !(1 << irq_index)); }
    }
}

pub fn disable() {
    unsafe { PICS.disable(); }
}

pub fn spurios_irq(from_master: bool) {
    if !from_master {
        send_eoi(PIC1_OFFSET);
    }
    SPURIOUS_IRQS_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn get_spurious_irqs_count() -> usize {
    SPURIOUS_IRQS_COUNT.load(Ordering::Relaxed)
}
