pub mod hpet;
pub mod vga;

use crate::memory::map_mmio_ptr;
pub fn update_mmio_with_paging() {
    let mut vga_guard = vga::VGA.lock();
    let vga = vga_guard
        .as_mut()
        .expect("VGA wasn't init before calling update with mmio!");
    vga.update_ptr(
        map_mmio_ptr(vga.get_ptr() as usize, vga.get_buffer_size()).expect("Mapping failed!")
            as *mut u16,
    );
}

pub fn uptime_nano() -> u64 {
    hpet::uptime_nano()
}

pub fn stall(nanos: u64) {
    let start = uptime_nano();
    while uptime_nano() < start + nanos { }
}
