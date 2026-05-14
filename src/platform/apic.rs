use core::sync::atomic::{AtomicUsize, Ordering};
use pic8259::ChainedPics;
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerDivide, TimerMode};

use crate::memory::map_mmio_ptr;

pub const SUPRIOUS_VECTOR: u8 = 0xFF;
pub const APIC_ERROR_VECTOR: u8 = 0xFE;
pub const TIMER_VECOTR: u8 = 0x30;
const LAPIC_MMIO_SIZE: usize = 0x1000;

pub static LAPIC_ADDR: AtomicUsize = AtomicUsize::new(0);

pub(super) fn set_lapic_addr(lapic_addr: usize) {
    LAPIC_ADDR.store(lapic_addr, Ordering::Relaxed);
}

pub fn init_lapic() -> LocalApic {
    disable_pic();
    let lapic_ptr =
        map_mmio_ptr(LAPIC_ADDR.load(Ordering::Relaxed), LAPIC_MMIO_SIZE).expect("MMIO map failed");
    let mut lapic = LocalApicBuilder::new()
        .set_xapic_base(lapic_ptr as u64)
        .spurious_vector(SUPRIOUS_VECTOR as usize)
        .error_vector(APIC_ERROR_VECTOR as usize)
        .timer_vector(TIMER_VECOTR as usize)
        .timer_mode(TimerMode::OneShot)
        .timer_divide(TimerDivide::Div16)
        .timer_initial(0)
        .build()
        .expect("Local APIC build failed");
    unsafe {
        lapic.enable();
    }
    lapic
}

pub fn init_lapic_timer(lapic: &mut LocalApic, nanos_per_int: u64) {
    const CALIBRATION_ITERATION_COUNT: u32 = 5;
    let mut ticks_sum = 0;
    unsafe {
        lapic.enable_timer();
    }
    for _ in 0..CALIBRATION_ITERATION_COUNT {
        unsafe {
            lapic.set_timer_initial(u32::MAX);
        }
        crate::utils::time::stall(nanos_per_int);
        ticks_sum += u32::MAX - unsafe { lapic.timer_current() };
    }
    let tick_avrg = ticks_sum / CALIBRATION_ITERATION_COUNT;
    unsafe {
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_initial(tick_avrg);
    }
}

fn disable_pic() {
    let mut pic = unsafe { ChainedPics::new(0x20, 0x28) };
    unsafe {
        pic.disable();
    }
}
