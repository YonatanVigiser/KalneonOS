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

#[derive(Debug, Clone)]
enum TerminalOutputState {
    Normal,
    Escape,
    EscapeSequence(CommandParams),
}

#[derive(Debug, Clone)]
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
            }
            let last_num = self.params[self.current_index];
            self.params[self.current_index] = last_num * 10 + next_char as u8 - AsciiChar::Num0 as u8;
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
    MoveCursorToColoumn(usize),
    MoveCursorTo(usize, usize),
    ClearScreenFromCursorToEnd,
    ClearScreenToCursor,
    ClearScreen,
    ClearLineFromCursorToEnd,
    ClearLineToCursor,
    ClearLine,
    ScrollUp(usize),
    ScrollDown(usize),
    SetColors(Option<Color>, Option<Color>),
}

#[derive(Debug, Clone)]
pub struct TerminalDataCell {
    ascii: AsciiChar,
    bg: Color,
    fg: Color,
}

#[derive(Debug, Clone)]
pub struct Terminal {
    cx: usize,
    cy: usize,
    scroll_lines: usize,
    bg: Color,
    fg: Color,
    height: usize,
    width: usize,
    data_buffer: Vec<Vec<TerminalDataCell>>,
    state: TerminalOutputState,
}

impl Terminal {
    fn new(height: usize, width: usize) -> Self {
        let data_buffer = vec![vec![
            TerminalDataCell {
                ascii: AsciiChar::Null,
                bg: Color::black(),
                fg: Color::white(),
            } ; width]; height];
        Self {
            cx: 0,
            cy: 0,
            scroll_lines: 0,
            bg: Color::black(),
            fg: Color::white(),
            height,
            width,
            data_buffer,
            state: TerminalOutputState::Normal,
        }
    }

    fn send(&mut self, driver: &mut TerminalDriver, data: AsciiChar) -> Result<(), ()> {
        let value = match &self.state {
            TerminalOutputState::Normal => {
                match data {
                    AsciiChar::Esc => {
                        self.state = TerminalOutputState::Escape;
                        Ok(())
                    },
                    _ => {
                        let result = match driver {
                            TerminalDriver::Video(video) => video.write_char(self.cx, self.cy, self.fg, self.bg, data),
                            TerminalDriver::Stream(stream) => stream.write_char(data as u8 as char).map_err(|_| ()),
                        };
                        if result.is_ok() {
                            let buffer_len = self.data_buffer.len();
                            let buffer_y = if buffer_len > self.height {
                                buffer_len - self.height + self.cy
                            } else {
                                self.cy
                            };

                            if let Some(row) = self.data_buffer.get_mut(buffer_y) && let Some(cell) = row.get_mut(self.cx) {
                                *cell = data as u8;
                            }
                            self.cx += 1;
                        }
                        result
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
                let mut state_clone = state.clone();
                if let Some(command) = state_clone.add(data)? {
                    match command {
                        OutputCommand::MoveCursorUp(n) => self.cy = self.cy.saturating_sub(n),
                        OutputCommand::MoveCursorDown(n) => self.cy += n,
                        OutputCommand::MoveCursorForward(n) => self.cx += n,
                        OutputCommand::MoveCursorBack(n) => self.cx = self.cx.saturating_sub(n),
                        OutputCommand::MoveCursorToBeginingDown(n) => {
                            self.cx = 0;
                            self.cy += n;
                        },
                        OutputCommand::MoveCursorToBeginingUp(n) => {
                            self.cx = 0;
                            self.cy = self.cy.saturating_sub(n);
                        },
                        OutputCommand::MoveCursorToColoumn(n) => self.cx = n,
                        OutputCommand::MoveCursorTo(x, y) => {
                            self.cx = x;
                            self.cy = y;
                        },
                        OutputCommand::ClearScreenFromCursorToEnd => self.clear_range(driver, self.cx, self.cy, self.width - 1, self.height - 1),
                        OutputCommand::ClearScreenToCursor => self.clear_range(driver, 0, 0, self.cx, self.cy),
                        OutputCommand::ClearScreen => self.clear_range(driver, 0, 0, self.width - 1, self.height - 1),
                        OutputCommand::ClearLineFromCursorToEnd => self.clear_range(driver, self.cx, self.cy, self.width - 1, self.cy),
                        OutputCommand::ClearLineToCursor => self.clear_range(driver, 0, self.cy, self.cx, self.cy),
                        OutputCommand::ClearLine => self.clear_range(driver, 0, self.cy, self.width - 1, self.cy),
                        OutputCommand::SetColors(bg, fg) => {
                            if let Some(fg) = fg {
                                self.fg = fg;
                            }
                            if let Some(bg) = bg {
                                self.bg = bg;
                            }
                            if fg.is_none() && bg.is_none() {
                                self.fg = Color::white();
                                self.bg = Color::black();
                            }
                        },
                        OutputCommand::ScrollUp(n) => self.scroll_up(&mut driver, self.height),
                        OutputCommand::ScrollDown(n) => self.scroll_down(&mut driver, self.height),
                    }
                    self.state = TerminalOutputState::Normal;
                } else {
                    self.state = TerminalOutputState::EscapeSequence(state_clone);
                }
                Ok(())
            }
        };
        if value.is_err() {
            return Err(());
        }
        if self.cx >= self.width {
            self.cx = 0;
            self.cy += 1;
        }
        if self.cy >= self.height {
            self.scroll_down(&mut driver, self.cy - self.height - self.scroll_lines + 1);
        }
        Ok(())
    }

    fn scroll_down(&mut self, driver: &mut TerminalDriver, lines: usize) {
        self.scroll_lines += lines;

    }

    pub fn redraw(&self, driver: &TerminalDriver) {
        let mut x = 0;
        let mut y = self.scroll_lines;
        loop {
            let value = ;
            let c = AsciiChar::try_from(value).unwrap_or(AsciiChar::Null);
            x += 1;
            if x == self.width {
                x = 0;
                y += 1;
            }
        }
    }

    fn clear_range(&mut self, mut driver: &mut TerminalDriver, start_x: usize, start_y: usize, mut end_x: usize, mut end_y: usize) {
        let mut x = start_x;
        let mut y = start_y;
        end_x = end_x.min(self.width - 1);
        end_y = end_y.min(self.height - 1);

        let buffer_len = self.data_buffer.len();
        let buffer_offset = if buffer_len > self.height {
            buffer_len - self.height - self.scroll_lines
        } else {
            0
        };

        loop {
            if let TerminalDriver::Video(video) = &mut driver {
                let _ = video.write_char(x, y, self.bg, self.bg, AsciiChar::Null);
            }

            let buffer_y = buffer_offset + y;
            if let Some(row_list) = self.data_buffer.get_mut(buffer_y) && let Some(value) = row_list.get_mut(x) {
                *value = 0;
            }

            if x == end_x && y == end_y {
                break;
            }
            x += 1;
            if x == self.width {
                x = 0;
                y += 1;
            }
        }
    }
}
