use crate::{gdt, interrupts, cpu_local, memory, drivers};
use x86_64::structures::paging::{FrameAllocator, PageTableFlags, Mapper, PageSize};
use x86_64::registers::control::Cr3;
use x2apic::lapic::LocalApic;
use acpi::platform::{ProcessorState, ProcessorInfo};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub static ACTIVE_PROCESSORS_COUNTER: AtomicUsize = AtomicUsize::new(0);
static BSP_FINISH: AtomicBool = AtomicBool::new(false);

#[repr(C)]
struct ApCoreData {
    stack_top_ptr: u64,
    l4_table: u32,
    acpi_processor_uid: u32,
}

pub unsafe fn start(lapic: &mut LocalApic, processor_info: &ProcessorInfo) -> ! {
    let code_frame = memory::FRAME_ALLOCATOR.lock().as_mut().expect("Frame allocator isn't init").allocate_frame().expect("Frame allocation failed");
    unsafe { memory::MAPPER.lock().as_mut().unwrap().identity_map(code_frame, PageTableFlags::PRESENT | PageTableFlags::GLOBAL, memory::FRAME_ALLOCATOR.lock().as_mut().expect("Frame allocator isn't init")).unwrap().flush(); }
    let dest = (code_frame.start_address().as_u64() + memory::HHDM_START) as *mut u8;
    let copy_start = &raw const ap_init;
    let copy_end = &raw const ap_init_end;
    let copy_size = (copy_end as u64 - copy_start as u64) as usize;
    assert!(copy_size <= memory::FrameSize::SIZE as usize, "trampoline is greater than one page");
    unsafe { core::ptr::copy_nonoverlapping(copy_start, dest, copy_size) };
    let ap_core_data = unsafe { &mut *((dest as usize + copy_size - size_of::<ApCoreData>()) as *mut ApCoreData) } as &mut ApCoreData;
    let cr3_value = Cr3::read_raw();
    debug_assert!(cr3_value.0.start_address().as_u64() < 0x1_0000_0000, "L4 page table frame above 4GB phys");
    ap_core_data.l4_table = cr3_value.0.start_address().as_u64() as u32 | cr3_value.1 as u32;
    for core in processor_info.application_processors.iter().filter(|p| matches!(p.state, ProcessorState::WaitingForSipi)) {
        log::info!("Trying to wake core {}", core.processor_uid);
        ap_core_data.acpi_processor_uid = core.processor_uid;
        let stack = memory::allocate(memory::bsp_stack_range().count() + 1, PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE).expect("Stack allocation failed");
        memory::MAPPER.lock().as_mut().unwrap().unmap(stack.start).expect("Stack page guard unmapping failed").1.flush(); // Unmap Stack guard
        let stack_top_ptr = stack.end.start_address().as_u64();
        ap_core_data.stack_top_ptr = stack_top_ptr;
        let start_vector = (code_frame.start_address().as_u64() >> 12) as u8;
        unsafe {
            lapic.send_init_ipi(core.local_apic_id);
            drivers::stall(10_000_000);
            lapic.send_sipi(start_vector, core.local_apic_id);
            drivers::stall(1_000_000);
            lapic.send_sipi(start_vector, core.local_apic_id);
        }
    }
    BSP_FINISH.store(true, Ordering::Release);
    ap_start(processor_info.boot_processor.processor_uid)
}

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn ap_start(acpi_processor_uid: u32) -> ! {
    unsafe {
        gdt::load();
    }
    let lapic = interrupts::init_local();
    while !BSP_FINISH.load(Ordering::Acquire) {
        core::hint::spin_loop()
    }
    let logical_id = ACTIVE_PROCESSORS_COUNTER.fetch_add(1, Ordering::Relaxed);
    cpu_local::init(acpi_processor_uid, logical_id, lapic);
    interrupts::enable();
    loop {
        log::info!("Hello from core: {}", logical_id);
    }
}

unsafe extern "C" {
    static ap_init: u8;
    static ap_init_end: u8;
}
