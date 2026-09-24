use core::fmt::{Error, Write};

use alloc::sync::Arc;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::RgbColor;
use multiboot2::{FramebufferTag, FramebufferType};

use crate::common::log::LogSink;
use crate::dev::lease::LeaseCell;
use crate::dev::registry::{DEVICE_REGISTRY, DeviceId};

use self::framebuffer::{FramebufferInfo, PixelEncoding};
use self::vga::VgaTextInfo;

pub mod framebuffer;
pub mod vga;

pub enum DisplayInfo {
    Graphics(FramebufferInfo),
    Text(VgaTextInfo),
}

impl From<&FramebufferTag> for DisplayInfo {
    fn from(value: &FramebufferTag) -> Self {
        match value.buffer_type().unwrap() {
            FramebufferType::RGB { red, green, blue } => Self::Graphics(FramebufferInfo {
                address: value.address() as usize,
                width: value.width(),
                height: value.height(),
                pitch: value.pitch(),
                bpp: value.bpp(),
                pixel_encoding: PixelEncoding::RGB { red, green, blue },
            }),
            FramebufferType::Indexed { palette } => Self::Graphics(FramebufferInfo {
                address: value.address() as usize,
                width: value.width(),
                height: value.height(),
                pitch: value.pitch(),
                bpp: value.bpp(),
                pixel_encoding: PixelEncoding::Indexed {
                    palette: palette
                        .iter()
                        .map(|c| Rgb888::new(c.red, c.green, c.blue))
                        .collect(),
                },
            }),
            FramebufferType::Text => Self::Text(VgaTextInfo {
                address: value.address() as usize,
                width: value.width(),
                height: value.height(),
                pitch: value.pitch(),
                bpp: value.bpp(),
            }),
        }
    }
}

pub struct Cell {
    pub c: char,
    pub fg: Rgb888,
    pub bg: Rgb888,
}

pub enum TextSurfaceError {
    OutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    UnsupportedChar {
        ch: char,
    },
}

pub trait TextSurface: Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn put(&mut self, x: u32, y: u32, cell: Cell) -> Result<(), TextSurfaceError>;
    fn scroll_up(&mut self, rows: u32);
    fn present(&mut self);
}

pub(super) fn init(display_info: &DisplayInfo) {
    match display_info {
        DisplayInfo::Graphics(info) => framebuffer::init(info),
        DisplayInfo::Text(info) => vga::init(info),
    }
    let display_consoles = DEVICE_REGISTRY
        .read()
        .ids_with_role::<dyn TextSurface>()
        .into_iter()
        .map(|id| DisplayConsole::new(id));
    for display_console in display_consoles {
        DEVICE_REGISTRY
            .write()
            .register::<dyn LogSink>(Arc::new(LeaseCell::new(display_console)));
    }
}

pub struct DisplayConsole {
    dev_id: DeviceId,
    cx: u32,
    cy: u32,
    fg: Rgb888,
    bg: Rgb888,
}

impl DisplayConsole {
    pub fn new(dev_id: DeviceId) -> Self {
        Self {
            dev_id,
            cx: 0,
            cy: 0,
            fg: Rgb888::WHITE,
            bg: Rgb888::BLACK,
        }
    }
}

impl Write for DisplayConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut dev = DEVICE_REGISTRY
            .read()
            .try_acquire::<dyn TextSurface>(self.dev_id)
            .ok_or(Error)?;
        for c in s.chars() {
            match c {
                '\n' => {
                    self.cy += 1;
                    self.cx = 0;
                }
                '\t' => self.cx += 4,
                c => {
                    dev.put(
                        self.cx,
                        self.cy,
                        Cell {
                            c,
                            fg: self.fg,
                            bg: self.bg,
                        },
                    )
                    .map_err(|_| Error)?;
                    self.cx += 1;
                }
            };
            if self.cx >= dev.width() {
                self.cx = 0;
                self.cy += 1;
            }
            let height = dev.height();
            if self.cy >= height {
                let rows = self.cy - height + 1;
                self.cx = 0;
                self.cy = dev.height() - 1;
                dev.scroll_up(rows);
            }
        }
        dev.present();
        Ok(())
    }
}
