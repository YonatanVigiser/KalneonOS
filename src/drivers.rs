pub mod vga;

pub fn init() {
    *vga::VGA.lock() = Some(vga::Vga::init(80, 25));
}
