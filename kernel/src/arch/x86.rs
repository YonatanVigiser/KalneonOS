pub mod cpu;
pub mod drivers;
pub mod idt;
pub mod interrupts;
pub mod pic;
pub mod heap;

use super::Arch;

use crate::drivers::traits::console::VideoConsole;

use drivers::pit::PitTimer;
use drivers::serial::SerialDriver;
use drivers::vga::Vga;

use core::fmt::Write;
use core::panic::PanicInfo;

use spin::mutex::Mutex;

// Early boot drivers references for IRQ handlers
static VIDEO_DRIVER: Mutex<Option<Vga>> = Mutex::new(None);
static SERIAL_DRIVER: Mutex<Option<SerialDriver>> = Mutex::new(None);
static TIMER_DRIVER: Mutex<Option<PitTimer>> = Mutex::new(None);

// Helper functions for IRQ-safe direct access (bypasses mutex, only for IRQ handlers)
impl ArchX86 {
    /// SAFETY: This must ONLY be called from IRQ handlers where interrupts are already disabled
    /// Accesses the driver data directly, bypassing the mutex lock to avoid deadlocks in IRQ context
    pub unsafe fn video_irq_unsafe() -> &'static mut Option<Vga> {
        // SAFETY: In IRQ context, interrupts are disabled, so we have exclusive access
        // We're directly accessing the data inside the Mutex by converting to raw pointer
        unsafe {
            let mutex_ptr = &VIDEO_DRIVER as *const Mutex<Option<Vga>> as *mut Mutex<Option<Vga>>;
            (*mutex_ptr).get_mut()
        }
    }

    /// SAFETY: This must ONLY be called from IRQ handlers where interrupts are already disabled
    pub unsafe fn serial_irq_unsafe() -> &'static mut Option<SerialDriver> {
        unsafe {
            let mutex_ptr = &SERIAL_DRIVER as *const Mutex<Option<SerialDriver>>
                as *mut Mutex<Option<SerialDriver>>;
            (*mutex_ptr).get_mut()
        }
    }

    /// SAFETY: This must ONLY be called from IRQ handlers where interrupts are already disabled
    pub unsafe fn timer_irq_unsafe() -> &'static mut Option<PitTimer> {
        unsafe {
            let mutex_ptr =
                &TIMER_DRIVER as *const Mutex<Option<PitTimer>> as *mut Mutex<Option<PitTimer>>;
            (*mutex_ptr).get_mut()
        }
    }
}

pub struct ArchX86();

impl Arch for ArchX86 {
    type VideoDriver = Vga;
    type SerialDriver = SerialDriver;
    type TimerDriver = PitTimer;

    fn init(_boot_magic_val: usize, _boot_info_ptr: usize) -> Self {
        // Init CPU
        idt::init();

        // Init heap
        heap::init_heap();

        // Init intterupts:
        pic::init();

        // Init early drivers
        *VIDEO_DRIVER.lock() = Some(Vga::init(80, 25));
        *SERIAL_DRIVER.lock() = SerialDriver::init();
        *TIMER_DRIVER.lock() = Some(PitTimer::init());

        // Finish init - enable interrupts
        unsafe {
            cpu::sti();
        }
        Self()
    }

    fn panic(&mut self, info: &PanicInfo) -> ! {
        use crate::kernel::display::color::Color;
        unsafe {
            cpu::cli();
        }

        if let Some(ref mut video_console) = *Self::video().lock() {
            video_console
                .set_bg(Color::red())
                .set_fg(Color::black())
                .clear();
            let _ = writeln!(video_console, "{}", info);
        }
        loop {}
    }

    fn video() -> &'static Mutex<Option<Vga>> {
        &VIDEO_DRIVER
    }

    fn serial() -> &'static Mutex<Option<SerialDriver>> {
        &SERIAL_DRIVER
    }

    fn timer() -> &'static Mutex<Option<PitTimer>> {
        &TIMER_DRIVER
    }
}
