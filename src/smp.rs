use crate::{gdt, interrupts, cpu_local, memory, drivers};
use x86_64::structures::paging::{FrameAllocator, PageTableFlags, Mapper};
use x86_64::registers::control::Cr3;
use x2apic::lapic::LocalApic;
use acpi::platform::{ProcessorInfo, ProcessorState};
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Once;

static AP_CORE_INIT_FINISH: AtomicBool = AtomicBool::new(false);
pub static PROCESSORS_INFO: Once<ProcessorInfo> = Once::new();

pub unsafe fn wake_all_ap_cores(lapic: &mut LocalApic, processor_info: &mut ProcessorInfo) {
    let mut frame_allocator_guard = memory::FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator_guard.as_mut().expect("Frame allocator isn't init");
    let code_frame = frame_allocator.allocate_frame().expect("Frame allocation failed");
    unsafe { memory::MAPPER.lock().as_mut().unwrap().identity_map(code_frame, PageTableFlags::PRESENT | PageTableFlags::GLOBAL, frame_allocator).unwrap().flush(); }
    let dest = (code_frame.start_address().as_u64() + memory::HHDM_START) as *mut u8;
    let copy_start = &raw const ap_init;
    let copy_end = &raw const ap_init_end;
    let copy_size = (copy_end as u64 - copy_start as u64) as usize;
    unsafe { core::ptr::copy_nonoverlapping(copy_start, dest, copy_size) };
    unsafe { core::ptr::write(copy_end as *mut u32, Cr3::read().0.start_address().as_u64() as u32); }
    for core in processor_info.application_processors.iter_mut().filter(|p| matches!(p.state, ProcessorState::WaitingForSipi)) {
        let stack = memory::allocate(memory::bsp_stack_range().count() + 1, PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE).expect("Stack allocation failed");
        memory::MAPPER.lock().as_mut().unwrap().unmap(stack.start).unwrap(); // Stack guard
        let stack_top_ptr = stack.end.start_address().as_u64();
        unsafe { core::ptr::write(copy_end.add(4) as *mut u64, stack_top_ptr); }
        let start_vector = (code_frame.start_address().as_u64() >> 12) as u8;
        unsafe {
            lapic.send_init_ipi(core.local_apic_id);
            drivers::stall(10_000_000);
            lapic.send_sipi(start_vector, core.local_apic_id);
            drivers::stall(200_000);
            if !AP_CORE_INIT_FINISH.load(Ordering::Acquire) {
                lapic.send_sipi(start_vector, core.local_apic_id);
                drivers::stall(200_000);
            }
            core.state = if AP_CORE_INIT_FINISH.swap(false, Ordering::AcqRel) { ProcessorState::Running } else { ProcessorState::Disabled };
        }
    }
    PROCESSORS_INFO.call_once(|| processor_info);
}

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn ap_start() -> ! {
    unsafe {
        gdt::load();
    }
    let lapic = interrupts::init_local();
    cpu_local::init(lapic);
    AP_CORE_INIT_FINISH.store(true, Ordering::Release);
    loop {
        core::hint::spin_loop();
    }
}

unsafe extern "C" {
    static ap_init: u8;
    static ap_init_end: u8;
}
