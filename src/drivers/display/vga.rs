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
const VGA_RGB_PALLETE: [Rgb888; 16] = [
    Rgb888::new(0, 0, 0),
    Rgb888::new(0, 0, 170),
    Rgb888::new(0, 170, 0),
    Rgb888::new(0, 170, 170),
    Rgb888::new(170, 0, 0),
    Rgb888::new(170, 0, 170),
    Rgb888::new(170, 85, 0),
    Rgb888::new(170, 170, 170),
    Rgb888::new(85, 85, 85),
    Rgb888::new(85, 85, 255),
    Rgb888::new(85, 255, 85),
    Rgb888::new(85, 255, 255),
    Rgb888::new(255, 85, 85),
    Rgb888::new(255, 85, 255),
    Rgb888::new(255, 255, 85),
    Rgb888::new(255, 255, 255),
];

#[derive(Debug, Copy, Clone)]
pub struct VgaCell {
    pub ascii: u8,
    pub bg: VgaColor,
    pub fg: VgaColor,
}

impl From<u16> for VgaCell {
    fn from(value: u16) -> Self {
        VgaCell {
            ascii: (value & 0x00FF) as u8,
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

impl TryFrom<Cell> for VgaCell {
    type Error = TextSurfaceError;
    fn try_from(value: Cell) -> Result<Self, Self::Error> {
        let ascii = value.c.as_ascii().ok_or(TextSurfaceError::UnsupportedChar { ch: value.c })?.into();
        let fg = (nearest_color(&VGA_RGB_PALLETE, value.fg) as u8).into();
        let bg = (nearest_color(&VGA_RGB_PALLETE, value.bg) as u8).into();
        Ok(VgaCell {
            ascii,
            fg,
            bg,
        })
    }
}

use alloc::sync::Arc;
use embedded_graphics::pixelcolor::Rgb888;
pub use spin::Mutex;

use crate::common::color::nearest_color;
use crate::dev::lease::LeaseCell;
use crate::dev::registry::DEVICE_REGISTRY;
use crate::memory::map_mmio_ptr;

use super::{Cell, TextSurface, TextSurfaceError};

#[derive(Debug, Clone)]
pub struct VgaTextInfo {
    pub address: usize,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
}

#[derive(Debug, Clone)]
pub struct Vga {
    info: VgaTextInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VgaError {
    NullAddress,
    ZeroSized,
    UnsupportedBpp(u8),
    PitchTooSmall { pitch: u32, required: u32 },
    OutOfBounds { x: u32, y: u32, width: u32, height: u32 }
}

impl Vga {
    pub fn new(info: &VgaTextInfo) -> Result<Self, VgaError> {
        if info.address == 0 {
            return Err(VgaError::NullAddress);
        }
        if info.width == 0 || info.height == 0 {
            return Err(VgaError::ZeroSized);
        }
        if info.bpp != 16 {
            return Err(VgaError::UnsupportedBpp(info.bpp));
        }
        let required = info.width * 2 as u32;
        if info.pitch < required {
            return Err(VgaError::PitchTooSmall {
                pitch: info.pitch,
                required,
            });
        }
        let framebuffer_size = info.pitch as usize * info.height as usize;

        let mapped_address = map_mmio_ptr(info.address, framebuffer_size).expect("Mapping failed failed");
        let mut info = info.clone();
        info.address = mapped_address;
        Ok(Self {
            info,
        })
    }

    fn put_cell(&self, x: u32, y: u32, cell: VgaCell) -> Result<(), VgaError> {
        if x >= self.info.width || y >= self.info.height {
            return Err(VgaError::OutOfBounds { x, y, width: self.width(), height: self.height() });
        }
        let value: u16 = cell.into();
        let index = (x as usize) + (y as usize) * (self.info.width as usize);
        let ptr = unsafe { (self.info.address as *mut u16).add(index) };
        unsafe { ptr.write_volatile(value) };
        Ok(())
    }

    fn get_cell(&self, x: u32, y: u32) -> Result<VgaCell, VgaError> {
        if x >= self.info.width || y >= self.info.height {
            return Err(VgaError::OutOfBounds { x, y, width: self.width(), height: self.height() });
        }
        let index = (x as usize) + (y as usize) * (self.info.width as usize);
        let ptr = unsafe { (self.info.address as *mut u16).add(index) };
        let value: u16 = unsafe { ptr.read_volatile() };
        Ok(value.into())
    }

    pub fn info(&self) -> &VgaTextInfo {
        &self.info
    }
}

impl TextSurface for Vga {
    fn width(&self) -> u32 {
        self.info.width
    }

    fn height(&self) -> u32 {
        self.info.height
    }

    fn put(&mut self, x: u32, y: u32, cell: Cell) -> Result<(), TextSurfaceError> {
        self.put_cell(x, y, cell.try_into()?).map_err(|_| TextSurfaceError::OutOfBounds { x, y, width: self.width(), height: self.height() })
    }

    fn scroll_up(&mut self, rows: u32) {
        for y in rows..self.height() {
            for x in 0..self.width() {
                self.put_cell(x, y - rows, self.get_cell(x, y).unwrap()).unwrap();
            }
        }
    }

    fn present(&mut self) {
    }
}

pub fn init(info: &VgaTextInfo) {
    let vga = Vga::new(info).expect("Vga init failed");
    let vga_dev = Arc::new(LeaseCell::new(vga));
    DEVICE_REGISTRY.write().register::<dyn TextSurface>(vga_dev);
}
