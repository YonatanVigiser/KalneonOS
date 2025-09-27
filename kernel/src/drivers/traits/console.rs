use crate::kernel::display::color::Color;

#[enum_dispatch::enum_dispatch]
pub trait Console: core::fmt::Write {
    fn clear(&mut self) -> &mut dyn Console;
    fn get_cursor_pos(&self) -> (usize, usize);
    fn move_cursor(&mut self, x: usize, y: usize) -> Result<&mut dyn Console, ()>;
    fn set_bg(&mut self, color: Color) -> &mut dyn Console;
    fn set_fg(&mut self, color: Color) -> &mut dyn Console;
    fn scroll_up(&mut self, by: usize) -> &mut dyn Console;
    fn scroll_down(&mut self, by: usize) -> &mut dyn Console;
}

#[enum_dispatch::enum_dispatch(Console)]
pub enum ConsoleImpl {
    Vga(crate::arch::x86::drivers::vga::Vga),
}

impl core::fmt::Write for ConsoleImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self {
            ConsoleImpl::Vga(inner) => inner.write_str(s),
        }
    }
}
