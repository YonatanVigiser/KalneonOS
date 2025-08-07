use core::ptr::{ read_volatile, write_volatile };

#[derive(Debug, Copy, Clone)]
pub enum VideoType {
  Color,
  Monochrome,
  None,
}

impl From<u8> for VideoType {
  fn from(value: u8) -> Self {
    match value & 0x30 {
      0x20 => VideoType::Color,
      0x30 => VideoType::Monochrome,
      _ => VideoType::None,
    }
  }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VgaColor {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGray = 7,
  DarkGray = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  LightMagenta = 13,
  Yellow = 14,
  White = 15,
}

impl TryFrom<u8> for VgaColor {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(VgaColor::Black),
      1 => Ok(VgaColor::Blue),
      2 => Ok(VgaColor::Green),
      3 => Ok(VgaColor::Cyan),
      4 => Ok(VgaColor::Red),
      5 => Ok(VgaColor::Magenta),
      6 => Ok(VgaColor::Brown),
      7 => Ok(VgaColor::LightGray),
      8 => Ok(VgaColor::DarkGray),
      9 => Ok(VgaColor::LightBlue),
      10 => Ok(VgaColor::LightGreen),
      11 => Ok(VgaColor::LightCyan),
      12 => Ok(VgaColor::LightRed),
      13 => Ok(VgaColor::LightMagenta),
      14 => Ok(VgaColor::Yellow),
      15 => Ok(VgaColor::White),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct VgaCell {
  pub ascii: char,
  pub bg: VgaColor,
  pub fg: VgaColor,
}

impl From<u16> for VgaCell {
  fn from(value: u16) -> Self {
    VgaCell {
      ascii: char::from((value & 0x00FF) as u8),
      bg: VgaColor::try_from(((value & 0xF000) >> 12) as u8).unwrap_or(VgaColor::Black),
      fg: VgaColor::try_from(((value & 0x0F00) >> 8) as u8).unwrap_or(VgaColor::White),
    }
  }
}

impl From<VgaCell> for u16 {
  fn from(cell: VgaCell) -> Self {
    (cell.bg as u16) << 12 | (cell.fg as u16) << 8 | cell.ascii as u16
  }
}

#[derive(Debug)]
pub enum VgaError {
  OutOfBoundsAccess,
  ConversionError,
}

pub struct VGA {
  vmem_ptr: *mut u16,
  video_type: VideoType,
  cx: u8,
  cy: u8,
  height: u8,
  width: u8,
  pub auto_scroll: bool,
  pub bg: VgaColor,
  pub fg: VgaColor,
  pub cursor_visible: bool,
  pub cursor_cell: VgaCell,
  cell_under_cursor: VgaCell,
}

// SAFETY: We ensure access to VGA is synchronized via `spin::Mutex`.
unsafe impl Send for VGA {}
unsafe impl Sync for VGA {}

impl VGA {
  pub fn new(width: u8, height: u8) -> Self {
    let video_type = get_video_type();
    let vmem_ptr = get_vmem_ptr(&video_type);
    Self {
      vmem_ptr,
      video_type,
      cx: 0,
      cy: 0,
      bg: VgaColor::Black,
      fg: VgaColor::White,
      height,
      width,
      auto_scroll: true,
      cursor_visible: true,
      cursor_cell: VgaCell {
        ascii: '_',
        bg: VgaColor::Black,
        fg: VgaColor::White,
      },
      cell_under_cursor: VgaCell {
        ascii: ' ',
        bg: VgaColor::Black,
        fg: VgaColor::White,
      },
    }
  }

  pub fn put_cell(&self, x: u8, y: u8, cell: VgaCell) -> Result<(), VgaError> {
    if x >= self.width || y >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    let value: u16 = cell.into();
    let index = (x as usize) + (y as usize) * (self.width as usize);
    let ptr = unsafe { self.vmem_ptr.add(index) };
    unsafe { ptr.write_volatile(value) };
    Ok(())
  }

  fn get_cell(&self, x: u8, y: u8) -> Result<VgaCell, VgaError> {
    if x >= self.width || y >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    let index = (x as usize) + (y as usize) * (self.width as usize);
    let ptr = unsafe { self.vmem_ptr.add(index) };
    let value: u16 = unsafe { ptr.read_volatile() };
    let cell: VgaCell = value.try_into().map_err(|_| VgaError::ConversionError)?;
    Ok(cell)
  }
  
  pub fn write_char(&mut self, c: char) -> Result<&mut Self, VgaError> {
    if self.cx >= self.width || self.cy >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    let mut new_cx = self.cx;
    let mut new_cy = self.cy;
    match c {
      '\0' => return Ok(self),
      '\n' => {
        new_cy += 1;
        new_cx = 0;
      },
      '\t' => {
        new_cx = (self.cx + 4) & !3;
      },
      _ => {
        self.cell_under_cursor = VgaCell {
          ascii: c,
          bg: self.bg,
          fg: self.fg,
        };
        new_cx += 1;
      },
    };
    if new_cx > self.width {
      new_cy += 1;
      new_cx = 0;
    }
    if new_cy == self.height && self.auto_scroll {
      let _ = self.scroll_down(1);
      let _ = self.move_cursor(0, self.height - 1);
    } else {
      self.move_cursor(new_cx, new_cy)?;
    }
    Ok(self)
  }

  pub fn write_string(&mut self, string: &str) -> Result<&mut Self, VgaError> {
    for b in string.bytes() {
      let _ = self.write_char(b as char)?;
    }
    Ok(self)
  }

  pub fn set_colors(&mut self, bg: VgaColor, fg: VgaColor) -> &mut Self {
    self.bg = bg;
    self.fg = fg;
    self.cursor_cell.bg = bg;
    self.cursor_cell.fg = fg;
    self
  }

  pub fn clear(&mut self) -> &mut Self {
    let empty_cell = VgaCell {
      ascii: ' ',
      bg: self.bg,
      fg: self.fg,
    };
    for index in 0..(self.height as usize * self.width as usize) {
      unsafe { self.vmem_ptr.add(index).write_volatile(empty_cell.into()); }
    }
    self.cell_under_cursor = empty_cell;
    let _ = self.move_cursor(0, 0);
    self
  }

  fn copy_line(&mut self, from: u8, to: u8) -> Result<(), VgaError> {
    for index in 0..self.width {
      let cell = self.get_cell(index, from)?;
      self.put_cell(index, to, cell)?;
    }
    Ok(())
  }

  fn clear_line(&mut self, line: u8) -> Result<(), VgaError> {
    for index in 0..self.width {
      self.put_cell(index, line, VgaCell {
        ascii: ' ',
        bg: self.bg,
        fg: self.fg,
      })?;
    }
    Ok(())
  }

  pub fn scroll_down(&mut self, amount: u8) -> &mut Self {
    if amount == 0 { return self; }
    if amount > self.height {
      self.clear();
    } else {
      for line_num in amount..self.height {
        let _ = self.copy_line(line_num, line_num - 1);
      }
    }
    let _ = self.clear_line(self.height - 1);
    if self.cy == 0 {
      let _ = self.move_cursor(0, 0);
    } else {
      self.cy -= 1;
    }
    self
  }

  pub fn scroll_up(&mut self, amount: u8) -> &mut Self {
    if amount == 0 { return self; }
    if amount > self.height {
      self.clear();
      let _ = self.move_cursor(0, 0);
    } else {
      for line_num in 0..(self.height - amount) {
        let _ = self.copy_line(line_num, line_num + 1);
      }
    }
    let _ = self.clear_line(0);
    self.cy += 1;
    if self.cy == self.height {
      let _ = self.move_cursor(0, self.height- 1);
    }
    self
  }

  pub fn move_cursor(&mut self, x: u8, y: u8) -> Result<&mut Self, VgaError> {
    if x >= self.width || y >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    self.put_cell(self.cx, self.cy, self.cell_under_cursor)?;
    self.cell_under_cursor = self.get_cell(x, y)?;
    if self.cursor_visible {
      self.put_cell(x, y, self.cursor_cell)?;
    }
    self.cx = x;
    self.cy = y;
    Ok(self)
  }

  pub fn update_cursor(&mut self) -> &mut Self {
    let _ = self.move_cursor(self.cx, self.cy);
    self
  }

  pub fn get_cursor_pos(&self) -> (u8, u8) {
    (self.cx, self.cy)
  }

  pub fn get_video_type(&self) -> VideoType {
    self.video_type
  }
}


use core::fmt;
impl fmt::Write for &mut VGA {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.write_string(s).map(|_| ()).map_err(|_| fmt::Error)
  }

  fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
    struct Adapter<'a>(&'a mut VGA);

    impl<'a> fmt::Write for Adapter<'a> {
      fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_string(s).map(|_| ()).map_err(|_| fmt::Error)
      }
    }

    let mut adapter = Adapter(self);

    args.as_str()
      .map(|s| adapter.write_str(s))
      .unwrap_or_else(|| Err(fmt::Error))
  }
}

fn get_video_type() -> VideoType {
  let bda_detected_hardware_ptr: *const u8 = 0x410 as *const u8;
  unsafe { bda_detected_hardware_ptr.read_volatile() }.into()
}

fn get_vmem_ptr(video_type: &VideoType) -> *mut u16 {
  match video_type {
    VideoType::Color => 0xB8000 as *mut u16,
    VideoType::Monochrome => 0xB0000 as *mut u16,
    VideoType::None => 0xB8000 as *mut u16, // Fake vmem
  }
}
