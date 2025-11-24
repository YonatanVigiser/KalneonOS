use super::super::ascii::AsciiChar;
use crate::drivers::traits::console::VideoConsole;
use crate::drivers::traits::console::InputConsole;
use super::color::Color;

use core::fmt::Write;

enum TerminalDriver<'a> {
    Video(&'a mut dyn VideoConsole),
    Stream(&'a mut dyn Write),
}

pub struct Terminal {
    cx: usize,
    cy: usize,
    bg: Color,
    fg: Color,
    height: usize,
    width: usize,
    data_buffer: &'static [&'static [u8]],
}

impl Terminal {
}
