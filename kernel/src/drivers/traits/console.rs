pub mod keyboard;

use crate::kernel::display::color::Color;

pub trait OutputConsole: core::fmt::Write + Sync + Send {}

pub trait InputConsole: Sync + Send {
    fn process_input(&mut self);
    fn read_byte(&mut self) -> Option<u8>;
    fn has_next_byte(&self) -> bool;
}

pub trait VideoConsole: OutputConsole {
    fn get_cursor_pos(&self) -> (usize, usize);
    fn clear(&mut self) -> &mut dyn VideoConsole;
    fn move_cursor(&mut self, x: usize, y: usize) -> Result<&mut dyn VideoConsole, ()>;
    fn set_bg(&mut self, color: Color) -> &mut dyn VideoConsole;
    fn set_fg(&mut self, color: Color) -> &mut dyn VideoConsole;
    fn scroll_up(&mut self, by: usize) -> &mut dyn VideoConsole;
    fn scroll_down(&mut self, by: usize) -> &mut dyn VideoConsole;
}

pub trait SerialConsole: InputConsole + OutputConsole {}
