pub mod x86;

use crate::drivers::traits::console::{SerialConsole, VideoConsole};
use crate::drivers::traits::console::keyboard::KeyboardDriver;
use crate::kernel::memory::map::MemoryMap;

use core::panic::PanicInfo;

pub trait Arch {
    fn init(boot_magic_val: usize, boot_info_ptr: usize);

    fn take_memory_map() -> Option<MemoryMap>;

    unsafe fn context_switch(old_stack_ptr: &mut usize, new_stack_ptr: usize);

    /*
    fn with_interrupts_disabled<F, R>(f: F) -> R
        where F: FnOnce() -> R;
    */

    fn fake_thread_entry_stack(stack_ptr: &mut usize, entry: fn() -> !);

    fn panic(info: &PanicInfo) -> !;

    fn with_keyboard<F, R>(f: F) -> Option<R> 
        where F: FnOnce(&mut dyn KeyboardDriver) -> R;

    fn with_serial<F, R>(f: F) -> Option<R> 
        where F: FnOnce(&mut dyn SerialConsole) -> R;

    fn with_video<F, R>(f: F) -> Option<R> 
        where F: FnOnce(&mut dyn VideoConsole) -> R;
}
