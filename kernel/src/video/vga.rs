use core::ptr::{ read_volatile, write_volatile };

#[derive(Clone)]
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

#[repr(u16)]
#[derive(Clone)]
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

#[derive(Clone)]
pub struct VgaCell {
  pub ascii: char,
  pub bg: VgaColor,
  pub fg: VgaColor,
}

impl TryFrom<u16> for VgaCell {
  type Error = ();

  fn try_from(value: u16) -> Result<Self, Self::Error> {
    Ok(VgaCell {
      ascii: char::from((value & 0x00FF) as u8),
      bg: VgaColor::try_from(((value & 0xF000) >> 12) as u8).map_err(|_| ())?,
      fg: VgaColor::try_from(((value & 0x0F00) >> 8) as u8).map_err(|_| ())?,
    })
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

impl VGA {
  pub fn new(width: u8, height: u8) -> Self {
    let video_type = get_video_type();
    let vmem_ptr = get_vmem_ptr(&VideoType::Color);
    let mut vga = VGA {
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
    };
    return vga;
  }

  pub fn put_cell(&self, x: u8, y: u8, cell: VgaCell) -> Result<(), VgaError> {
    if x >= self.width || y >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    let value: u16 = cell.into();
    let mut ptr: *mut u16 = self.vmem_ptr;
    let index = (x as usize) + (y as usize) * (self.width as usize);
    ptr = unsafe { ptr.add(index) };
    unsafe { ptr.write_volatile(value) };
    Ok(())
  }

  fn get_cell(&self, x: u8, y: u8) -> Result<VgaCell, VgaError> {
    if x >= self.width || y >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    let mut value: u16 = 0x0F41;
    let mut ptr: *mut u16 = self.vmem_ptr;
    let index = (x as usize) + (y as usize) * (self.width as usize);
    ptr = unsafe { ptr.add(index) };
    value = unsafe { ptr.read_volatile() };
    let cell: VgaCell = value.try_into().map_err(|_| VgaError::ConversionError)?;
    Ok(cell)
  }

  pub fn write_char(&mut self, c: char) -> Result<(), VgaError> {
    if self.cx >= self.width || self.cy >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    let mut new_cx = self.cx;
    let mut new_cy = self.cy;
    match c {
      '\0' => return Ok(()),
      '\n' => {
        new_cy += 1;
        new_cx = 0;
      },
      '\t' => {
        new_cx += 4;
      },
      _ => {
        self.cell_under_cursor = VgaCell {
          ascii: c,
          bg: self.bg.clone(),
          fg: self.fg.clone(),
        };
        new_cx += 1;
      },
    };
    if new_cx > self.width {
      new_cy += 1;
      new_cx = 0;
    }
    if new_cy >= self.height && self.auto_scroll {
      self.scroll_down(1)?;
      self.move_cursor(0, self.height - 1)?;
    } else {
      self.move_cursor(new_cx, new_cy)?;
    }
    Ok(())
  }

  pub fn write_string(&mut self, string: &str) -> Result<(), VgaError> {
    for b in string.bytes() {
      self.write_char(b as char)?;
    }
    Ok(())
  }

  pub fn clear_with_color(&mut self, bg: VgaColor, fg: VgaColor) {
    self.bg = bg.clone();
    self.fg = fg.clone();
    self.cursor_cell.bg = bg;
    self.cursor_cell.fg = fg;
    self.clear();
  }

  pub fn clear(&mut self) {
    let empty_cell = VgaCell {
      ascii: ' ',
      bg: self.bg.clone(),
      fg: self.fg.clone(),
    };
    for y in 0..self.height {
      for x in 0..self.width {
        let _ = self.put_cell(x, y, empty_cell.clone());
      }
    }
    self.cell_under_cursor = empty_cell;
    let _ = self.move_cursor(0, 0);
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
        bg: self.bg.clone(),
        fg: self.fg.clone(),
      })?;
    }
    Ok(())
  }

  pub fn scroll_down(&mut self, amount: u8) -> Result<(), VgaError> {
    if amount > self.height {
      self.clear();
    } else {
      for line_num in amount..self.height {
        self.copy_line(line_num, line_num - 1)?;
      }
    }
    self.clear_line(self.height - 1)?;
    if self.cy == 0 {
      self.move_cursor(0, 0)?;
    } else {
      self.cy -= 1;
    }
    Ok(())
  }

  pub fn scroll_up(&mut self, amount: u8) -> Result<(), VgaError> {
    if amount > self.height {
      self.clear();
      self.move_cursor(0, 0)?;
    } else {
      for line_num in 0..(self.height - amount) {
        self.copy_line(line_num, line_num + 1)?;
      }
    }
    self.clear_line(0);
    self.cy += 1;
    if self.cy == self.height {
      self.move_cursor(0, self.height- 1)?;
    }
    Ok(())
  }

  pub fn move_cursor(&mut self, x: u8, y: u8) -> Result<(), VgaError> {
    if x >= self.width || y >= self.height {
      return Err(VgaError::OutOfBoundsAccess);
    }
    self.put_cell(self.cx, self.cy, self.cell_under_cursor.clone())?;
    self.cell_under_cursor = self.get_cell(x, y)?;
    if self.cursor_visible {
      self.put_cell(x, y, self.cursor_cell.clone())?;
    }
    self.cx = x;
    self.cy = y;
    Ok(())
  }

  pub fn update_cursor(&mut self) {
    let _ = self.move_cursor(self.cx, self.cy);
  }

  pub fn get_cursor_pos(&self) -> (u8, u8) {
    (self.cx, self.cy)
  }

  pub fn get_video_type(&self) -> VideoType {
    self.video_type.clone()
  }
}

impl core::fmt::Write for VGA {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    let _ = self.write_string(s);
    Ok(())
  }
}

fn get_video_type() -> VideoType {
  let bda_detected_hardware_ptr: *const u8 = 0x410 as *const u8;
  let value: VideoType = unsafe { bda_detected_hardware_ptr.read_volatile() }.into();
  return value
}

fn get_vmem_ptr(video_type: &VideoType) -> *mut u16 {
  match video_type {
    VideoType::Color => 0xB8000 as *mut u16,
    VideoType::Monochrome => 0xB0000 as *mut u16,
    VideoType::None => 0xB8000 as *mut u16, // Fake vmem
  }
}
