#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(local_waker)]
#![feature(abi_x86_interrupt)]
#![feature(unsafe_cell_access)]

pub mod drivers;
pub mod interrupt;
pub mod memory;
pub mod platform;
pub mod task;
pub mod time;
pub mod utils;

extern crate alloc;

use core::sync::atomic::Ordering;

#[unsafe(link_section = ".multiboot")]
#[used]
static MULTIBOOT_HEADER: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/multiboot_header.bin"))
    .len()] = *include_bytes!(concat!(env!("OUT_DIR"), "/multiboot_header.bin"));

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn main(boot_magic: u32, boot_info_ptr: u32) -> ! {
    utils::log::init_boot_logger();
    let boot_info = platform::boot::load(boot_magic, boot_info_ptr);

    memory::init(&boot_info.mmap, || {
        let mut guard = drivers::display::vga::VGA.lock();
        let vga = guard.as_mut().expect("VGA not initialized before paging");
        let ptr = memory::map_mmio_ptr(vga.get_ptr() as usize, vga.get_buffer_size())
            .expect("VGA MMIO remap failed") as *mut u16;
        vga.update_ptr(ptr);
    });
    let acpi = platform::acpi::platform_info(boot_info.rsdt_addr, boot_info.rsdt_revision);
    time::hpet::init_hpet(&acpi.tables).expect("HPET init failed");
    interrupt::init_global(&acpi.interrupt_model);
    platform::acpi::ACPI.call_once(|| acpi);
    let lapic = interrupt::init_local();
    let processor_info = platform::acpi::ACPI
        .get()
        .unwrap()
        .processor_info
        .as_ref()
        .expect("No processor info in ACPI tables!");
    let bsp_logical_id = platform::smp::ACTIVE_PROCESSORS_COUNTER.fetch_add(1, Ordering::Relaxed);
    platform::cpu::init(
        processor_info.boot_processor.processor_uid,
        bsp_logical_id,
        lapic,
    );
    log::info!("Starting SMP...");
    let lapic = &mut platform::cpu::current_cpu().lapic;
    unsafe {
        platform::smp::start(lapic, processor_info);
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    halt_loop()
}

pub fn halt_loop() -> ! {
    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation failed: {:?}", layout)
}
