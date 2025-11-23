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

use crate::kernel::io::display::color::{Color, common::VGA_COLOR_PALLETE};

impl From<Color> for VgaColor {
    fn from(value: Color) -> Self {
        (VGA_COLOR_PALLETE.get_closest_index(&value) as u8).into()
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

pub struct Vga {
    vmem_ptr: *mut u16,
    video_type: VideoType,
    height: usize,
    width: usize,
}

// SAFETY: Vga contains a pointer to memory-mapped I/O (VGA text buffer at 0xB8000).
// This is not heap memory and is safe to access from any execution context.
// The hardware handles concurrent access, and VGA text mode operations are atomic at the u16 level.
unsafe impl Send for Vga {}
unsafe impl Sync for Vga {}

impl Vga {
    pub fn init(width: usize, height: usize) -> Self {
        let video_type = Self::get_video_type_bda();
        let vmem_ptr = Self::get_vmem_ptr(&video_type);
        Self {
            vmem_ptr,
            video_type,
            height,
            width,
        }
    }

    fn put_cell(&self, x: usize, y: usize, cell: VgaCell) -> Result<(), ()> {
        if x >= self.width || y >= self.height {
            return Err(());
        }
        let value: u16 = cell.into();
        let index = (x as usize) + (y as usize) * (self.width as usize);
        let ptr = unsafe { self.vmem_ptr.add(index) };
        unsafe { ptr.write_volatile(value) };
        Ok(())
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
}

use crate::drivers::traits::console::VideoConsole;
use crate::kernel::io::ascii::AsciiChar;

impl VideoConsole for Vga {
    fn write_char(&mut self, x: usize, y: usize, ascii_char: AsciiChar, bg: Color, fg: Color) -> Result<(), ()> {
        let cell = VgaCell { ascii: ascii_char as u8 as char, bg: bg.into(), fg: fg.into() };
        self.put_cell(x, y, cell)?;
        Ok(())
    }

    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.height
    }
}
