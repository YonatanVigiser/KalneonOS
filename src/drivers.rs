pub mod vga;

pub fn init() {
    *vga::VGA.lock() = Some(vga::Vga::init(80, 25));
}

pub fn update_mmio_with_paging() {
    let mut vga = vga::VGA.lock();
    let old_ptr = vga.as_ref().unwrap().get_ptr() as u64;
    vga.as_mut().unwrap().update_ptr((old_ptr + crate::memory::HHDM_START) as *mut u16);
}
