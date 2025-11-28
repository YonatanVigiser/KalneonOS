use super::super::ascii::AsciiChar;
use crate::drivers::traits::console::VideoConsole;
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
    EscapeSequence(CommandParams),
}

pub struct CommandParams {
    params: Vec<u8>,
    current_index: usize,
}

impl CommandParams {
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add(&mut self, next_char: AsciiChar) -> Result<Option<OutputCommand>, ()> {
        if let AsciiChar::Semicolon = next_char {
            self.current_index += 1;
            return Ok(None);
        }
        if next_char.is_numeric() {
            if self.current_index >= self.params.len()  {
                self.params.push(0);
            } else {
                let last_num = self.params.get(self.current_index).expect("This shouldn't fail!");
                self.params[self.current_index] = last_num * 10 + next_char as u8 - AsciiChar::Num0 as u8;
            }
            return Ok(None);
        }
        if matches!(next_char, AsciiChar::TildeSign) | next_char.is_alphabetic() {
            return match next_char {
                AsciiChar::A => Ok(Some(OutputCommand::MoveCursorUp(*self.params.get(0).ok_or(())? as usize))),
                _ => Err(()),
            };
        }
        Err(())
    }
}

enum OutputCommand {
    MoveCursorUp(usize),
    MoveCursorDown(usize),
    MoveCursorForward(usize),
    MoveCursorBack(usize),
    MoveCursorToBeginingDown(usize),
    MoveCursorToBeginingUp(usize),
    CursorMoveToColoumn(usize),
    MoveCursorTo(usize, usize),
    ClearScreenFromCursorToEnd,
    ClearScreenToCursor,
    ClearScreen,
    ClearLineFromCursorToEnd,
    ClearLineToCursor,
    ClearLine,
    ScrollUpOnePage,
    ScrollDownOnePage,
    SetColors(Option<usize>, Option<usize>),
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

    fn send(&mut self, driver: TerminalDriver, data: AsciiChar) -> Result<(), ()> {
        match self.state {
            TerminalOutputState::Normal => {
                match data {
                    AsciiChar::Esc => {
                        self.state = TerminalOutputState::Escape;
                        Ok(())
                    },
                    _ => match driver {
                            TerminalDriver::Video(video) => video.write_char(self.cx, self.cy, self.fg, self.bg, data),
                            TerminalDriver::Stream(stream) => stream.write_char(data as u8 as char).map_err(|_| ()),
                        }
                }
            },
            TerminalOutputState::Escape => {
                match data {
                    AsciiChar::OpeningBrackets => {
                        self.state = TerminalOutputState::EscapeSequence(CommandParams::new());
                        Ok(())
                    }
                    _ => Err(()),
                }
            },
            TerminalOutputState::EscapeSequence(state) => {
                if let Some(command) = state.add(data)? {
                    match command {
                        OutputCommand::MoveCursorUp(n) => self.cy += n,
                        OutputCommand::MoveCursorDown(n) => self.cy -= n,
                        OutputCommand::MoveCursorForward(n) => self.cx += n,
                        OutputCommand::MoveCursorBack(n) => self.cx -= n,
                        OutputCommand::MoveCursorToBeginingDown(n) => {
                            self.cx = 0;
                            self.cy += n;
                        },
                        OutputCommand::MoveCursorToBeginingUp(n) => {
                            self.cx = 0;
                            self.cy -= n;
                        },
                        OutputCommand::MoveCursorTo(x, y) => {
                            self.cx = x;
                            self.cy = y;
                        },
                        OutputCommand::ClearScreenFromCursorToEnd => {
                            clear_range(driver, self.cx, self.cy, self.height - 1, self.width - 1);
                        }
                    }
                    self.state = TerminalOutputState::Normal;
                }
                Ok(())
            }
        }
    }

    fn clear_range(driver: TerminalDriver, start_x: usize, start_y: usize, end_x: usize, end_y: usize) => {
        match driver 
    }
}
