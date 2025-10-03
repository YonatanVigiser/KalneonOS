use crate::kernel::display::color::Color;

pub trait OutputConsole: core::fmt::Write {}

pub trait InputConsole {
    fn read_event(&self) -> Option<u8>;
    fn has_event(&self) -> bool;
}

#[enum_dispatch::enum_dispatch]
pub trait VideoConsole: OutputConsole {
    fn get_cursor_pos(&self) -> (usize, usize);
    fn clear(&mut self) -> &mut dyn VideoConsole;
    fn move_cursor(&mut self, x: usize, y: usize) -> Result<&mut dyn VideoConsole, ()>;
    fn set_bg(&mut self, color: Color) -> &mut dyn VideoConsole;
    fn set_fg(&mut self, color: Color) -> &mut dyn VideoConsole;
    fn scroll_up(&mut self, by: usize) -> &mut dyn VideoConsole;
    fn scroll_down(&mut self, by: usize) -> &mut dyn VideoConsole;
}

#[enum_dispatch::enum_dispatch(VideoConsole)]
pub enum VideoConsoleImpl {
    Vga(crate::arch::x86::drivers::vga::Vga),
}

impl OutputConsole for VideoConsoleImpl {}

impl core::fmt::Write for VideoConsoleImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self {
            VideoConsoleImpl::Vga(inner) => inner.write_str(s),
        }
    }
}

#[enum_dispatch::enum_dispatch]
pub trait SerialConsole: InputConsole + OutputConsole {}

#[enum_dispatch::enum_dispatch(SerialConsole)]
pub enum SerialConsoleImpl {
    X86(crate::arch::x86::drivers::serial::SerialDriver),
}

impl OutputConsole for SerialConsoleImpl {}

impl core::fmt::Write for SerialConsoleImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self {
            SerialConsoleImpl::X86(inner) => inner.write_str(s),
        }
    }
}
