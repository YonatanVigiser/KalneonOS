use super::super::ascii::AsciiChar;
use crate::drivers::traits::console::VideoConsole;
use crate::drivers::traits::console::InputConsole;
use super::color::Color;

use alloc::vec;
use alloc::vec::Vec;

use core::fmt::Write;

enum TerminalDriver<'a> {
    Video(&'a mut dyn VideoConsole),
    Stream(&'a mut dyn Write),
}

enum TerminalOutputState {
    Normal,
    Escape,
    EscapeSequence,
    NumberRecivedAfterEscape(u8),
    SemicolonRecivedAfterEscape(u8),
    SecondParameterRecived(u8, u8),
}


pub struct Terminal {
    cx: usize,
    cy: usize,
    bg: Color,
    fg: Color,
    height: usize,
    width: usize,
    data_buffer: Vec<Vec<u8>>,
    state: TerminalOutputState,
}

use super::ascii:AsciiChar;
impl Terminal {
    fn new(height: usize, width: usize) -> Self {
        let data_buffer = vec![vec![0; width]; height];
        Self {
            cx: 0,
            cy: 0,
            bg: Color::black(),
            fg: Color::white(),
            height,
            width,
            data_buffer,
            state: TerminalOutputState::Normal,
        }
    }

    fn send(&mut self, driver: TerminalDriver, data: AsciiChar) {
        match data {
            AsciiChar::Esc => {
                if let TerminalOutputState::Normal = self.state {
                    self.state = TerminalOutputState::Escape;
                } else {
                    self.state = TerminalOutputState::Normal
                }
            }

        }
    }
}
