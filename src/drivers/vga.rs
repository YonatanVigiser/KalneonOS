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

impl From<u8> for VgaColor {
    fn from(value: u8) -> Self {
        match value {
            0 => VgaColor::Black,
            1 => VgaColor::Blue,
            2 => VgaColor::Green,
            3 => VgaColor::Cyan,
            4 => VgaColor::Red,
            5 => VgaColor::Magenta,
            6 => VgaColor::Brown,
            7 => VgaColor::LightGray,
            8 => VgaColor::DarkGray,
            9 => VgaColor::LightBlue,
            10 => VgaColor::LightGreen,
            11 => VgaColor::LightCyan,
            12 => VgaColor::LightRed,
            13 => VgaColor::LightMagenta,
            14 => VgaColor::Yellow,
            15 => VgaColor::White,
            _ => VgaColor::Black,
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
            bg: VgaColor::from(((value & 0xF000) >> 12) as u8),
            fg: VgaColor::from(((value & 0x0F00) >> 8) as u8),
        }
    }
}

impl From<VgaCell> for u16 {
    fn from(cell: VgaCell) -> Self {
        (cell.bg as u16) << 12 | (cell.fg as u16) << 8 | cell.ascii as u16
    }
}

pub use spin::Mutex;

pub static VGA: Mutex<Option<Vga>> = Mutex::new(None);

pub struct Vga {
    vmem_ptr: *mut u16,
    cx: u8,
    cy: u8,
    height: u8,
    width: u8,
    auto_scroll: bool,
    bg: VgaColor,
    fg: VgaColor,
    cursor_visible: bool,
    cursor_cell: VgaCell,
    cell_under_cursor: VgaCell,
}

// SAFETY: Vga contains a pointer to memory-mapped I/O (VGA text buffer at 0xB8000).
// This is not heap memory and is safe to access from any execution context.
// The hardware handles concurrent access, and VGA text mode operations are atomic at the u16 level.
unsafe impl Send for Vga {}
unsafe impl Sync for Vga {}

impl Vga {
    pub fn init(width: u8, height: u8) -> Self {
        let video_type = Self::get_video_type_bda();
        let vmem_ptr = Self::get_vmem_ptr(&video_type);
        let mut vga = Self {
            vmem_ptr,
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
        vga.clear();
        vga
    }

    fn put_cell(&self, x: u8, y: u8, cell: VgaCell) -> Result<(), ()> {
        if x >= self.width || y >= self.height {
            return Err(());
        }
        let value: u16 = cell.into();
        let index = (x as usize) + (y as usize) * (self.width as usize);
        let ptr = unsafe { self.vmem_ptr.add(index) };
        unsafe { ptr.write_volatile(value) };
        Ok(())
    }

    fn get_cell(&self, x: u8, y: u8) -> Result<VgaCell, ()> {
        if x >= self.width || y >= self.height {
            return Err(());
        }
        let index = (x as usize) + (y as usize) * (self.width as usize);
        let ptr = unsafe { self.vmem_ptr.add(index) };
        let value: u16 = unsafe { ptr.read_volatile() };
        Ok(value.into())
    }

    fn write_char(&mut self, c: char) -> Result<&mut Self, ()> {
        if self.cx >= self.width || self.cy >= self.height {
            return Err(());
        }
        let mut new_cx = self.cx;
        let mut new_cy = self.cy;
        match c {
            '\0' => return Ok(self),
            '\n' => {
                new_cy += 1;
                new_cx = 0;
            }
            '\t' => {
                new_cx = (self.cx + 4) & !3;
            }
            _ => {
                self.cell_under_cursor = VgaCell {
                    ascii: c,
                    bg: self.bg,
                    fg: self.fg,
                };
                new_cx += 1;
            }
        };
        if new_cx >= self.width {
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

    fn write_string(&mut self, string: &str) -> Result<&mut Self, ()> {
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
        self.update_cursor();
        self
    }

    fn copy_line(&mut self, from: u8, to: u8) -> Result<(), ()> {
        for index in 0..self.width {
            let cell = self.get_cell(index, from)?;
            self.put_cell(index, to, cell)?;
        }
        Ok(())
    }

    fn clear_line(&mut self, line: u8) -> Result<(), ()> {
        for index in 0..self.width {
            self.put_cell(
                index,
                line,
                VgaCell {
                    ascii: ' ',
                    bg: self.bg,
                    fg: self.fg,
                },
            )?;
        }
        let _= self.move_cursor(0, 0);
        Ok(())
    }

    pub fn move_cursor(&mut self, x: u8, y: u8) -> Result<&mut Self, ()> {
        if x >= self.width || y >= self.height {
            return Err(());
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

    fn get_video_type_bda() -> VideoType {
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

    pub fn clear(&mut self) -> &mut Self {
        let empty_cell = VgaCell {
            ascii: ' ',
            bg: self.bg,
            fg: self.fg,
        };
        for index in 0..(self.height as usize * self.width as usize) {
            unsafe {
                self.vmem_ptr.add(index).write_volatile(empty_cell.into());
            }
        }
        self.cell_under_cursor = empty_cell;
        let _ = self.move_cursor(0, 0);
        self
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        (self.cx as usize, self.cy as usize)
    }

    pub fn set_bg(&mut self, color: VgaColor) -> &mut Self {
        self.set_colors(color, self.fg);
        self
    }

    pub fn set_fg(&mut self, color: VgaColor) -> &mut Self {
        self.set_colors(self.bg, color);
        self
    }

    pub fn scroll_down(&mut self, amount: usize) -> &mut Self {
        if amount == 0 {
            return self;
        }
        if amount > self.height.into() {
            self.clear();
        } else {
            for line_num in (amount as u8)..self.height {
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

    pub fn scroll_up(&mut self, amount: usize) -> &mut Self {
        if amount == 0 {
            return self;
        }
        if amount > self.height.into() {
            self.clear();
            let _ = self.move_cursor(0, 0);
        } else {
            for line_num in 0..(self.height - amount as u8) {
                let _ = self.copy_line(line_num, line_num + 1);
            }
        }
        let _ = self.clear_line(0);
        self.cy += 1;
        if self.cy == self.height {
            let _ = self.move_cursor(0, self.height - 1);
        }
        self
    }

    pub fn get_ptr(&self) -> *mut u16 {
        self.vmem_ptr
    }
    
    pub fn update_ptr(&mut self, new_ptr: *mut u16) {
        self.vmem_ptr = new_ptr;
    }

    pub fn get_buffer_size(&self) -> usize {
        (self.width * self.height * u16::BITS as u8) as usize
    }
}

impl core::fmt::Write for Vga {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s)
            .map(|_| ())
            .map_err(|_| core::fmt::Error)
    }
}

