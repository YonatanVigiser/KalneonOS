use crate::arch::x86::{cpu, idt, pic};
use crate::boot::boot_info::BootInfoBlock;
use crate::drivers::pit;
use crate::drivers::video::vga::{VGA, VgaColor};
use crate::utils::types::SyncUnsafeCell;
use core::cell::UnsafeCell;
use core::fmt::Write;
use core::panic::PanicInfo;

use super::timer;

pub const TERMINAL_WIDTH: u8 = 80;
pub const TERMINAL_HEIGHT: u8 = 25;

pub static VGA_GLOBAL: SyncUnsafeCell<VGA> = SyncUnsafeCell(UnsafeCell::new(VGA::default()));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let vga = VGA_GLOBAL.get_mut();
    let _ = vga.set_colors(VgaColor::Red, VgaColor::Black).clear();
    let _ = write!(vga, "Kernel panicked: {info}");
    unsafe { cpu::cli() }
    loop {}
}

pub fn kernel_main(boot_info_ptr: u32) -> ! {
    unsafe { cpu::cli(); }
    let boot_info = unsafe { BootInfoBlock::copy_from_ptr(boot_info_ptr) };
    *VGA_GLOBAL.get_mut() = VGA::new(TERMINAL_WIDTH, TERMINAL_HEIGHT);
    idt::init();
    pic::init();
    timer::init();
    use pit::ChannelNum::C0;
    let vga = VGA_GLOBAL.get_mut();
    pit::set_reload_value(C0, 11932);
    let test = pit::get_count(C0);
    vga.set_colors(VgaColor::Black, VgaColor::White).clear();
    writeln!(vga, "Count: {test}");
    pic::unmask_irq(0);
    unsafe {
        cpu::sti();
    }
    writeln!(vga, "Old: {}", timer::get_uptime_ms()).expect("Error while writing to VGA!");
    timer::sleep(30000);
    writeln!(vga, "New: {}", timer::get_uptime_ms()).expect("Error while writing to VGA!");
    loop {}
}
