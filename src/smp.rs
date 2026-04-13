use crate::{gdt, interrupts, cpu_local, memory, drivers};
use x86_64::structures::paging::{FrameAllocator, PageTableFlags};
use x2apic::lapic::LocalApic;

pub unsafe fn wake_all_ap_cores(lapic: &mut LocalApic) {
    let mut frame_allocator_lock = memory::FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator_lock.as_mut().expect("Frame allocator isn't init");
    let code_frame = frame_allocator.allocate_frame().expect("Frame allocation failed");
    let code_ptr = code_frame.start_address().as_u64() as *mut u8;
    code_ptr.write f
    let stack = memory::allocate(memory::bsp_stack_range().count(), PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE).expect("Stack allocation failed");
    let stack_top_ptr = stack.end.start_address().as_u64();
    let start_vector = 0u8;
    unsafe {
        lapic.send_init_ipi_all();
        drivers::stall(10_000_000);
        lapic.send_sipi_all(start_vector);
        drivers::stall(200_000);
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn ap_start() -> ! {
    unsafe {
        gdt::load();
    }
    unsafe { memory::paging::enable(); }
    let lapic = interrupts::init_local();
    cpu_local::init(lapic);
    loop {
        core::hint::spin_loop();
    }
}

unsafe extern "C" {
    static ap_init: u8;
    static ap_init_end: u8;
}
