use crate::kernel::display::color::Color;

pub trait Console: core::fmt::Write {
    fn clear(&mut self) -> &mut Self;
    fn get_cursor_pos(&self) -> (usize, usize);
    fn move_cursor(&mut self, x: usize, y: usize) -> &mut Self;
    fn set_bg(&mut self, color: &Color) -> &mut Self;
    fn set_fg(&mut self, color: &Color) -> &mut Self;
    fn scroll_up(&mut self, by: usize) -> &mut Self;
    fn scroll_down(&mut self, by: usize) -> &mut Self;
}

#[enum_dispatch::enum_dispatch]
pub enum ConsoleImpl {
    Vga(crate::arch::x86::drivers::vga::Vga),
}
