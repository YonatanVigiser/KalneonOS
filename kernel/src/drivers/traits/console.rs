pub mod keyboard;

use crate::kernel::io::display::color::Color;
use crate::kernel::io::ascii::AsciiChar;

pub trait InputConsole: Sync + Send {
    fn process_input(&mut self);
    fn read_byte(&mut self) -> Option<u8>;
    fn has_next_byte(&self) -> bool;
}

pub trait VideoConsole: Sync + Send {
    fn write_char(&mut self, x: usize, y: usize, bg: Color, fg: Color, ascii_char: AsciiChar) -> Result<(), ()>;
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
}

use core::fmt::Write;
pub trait SerialConsole: Sync + Send + Write + InputConsole {}
