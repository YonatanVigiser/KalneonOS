pub mod vga;

pub fn init() {
    *vga::VGA.lock() = Some(vga::Vga::init(80, 25));
}

use crate::memory::map_mmio_ptr;
pub fn update_mmio_with_paging() {
    let mut vga_guard = vga::VGA.lock();
    let vga = vga_guard.as_mut().expect("VGA wasn't init before calling update with mmio!");
    vga.update_ptr(map_mmio_ptr(vga.get_ptr() as u64, vga.get_buffer_size() as u64).expect("Mapping failed!") as *mut u16);
}
