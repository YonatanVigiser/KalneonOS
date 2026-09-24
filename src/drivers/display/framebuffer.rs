use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{DrawTarget, OriginDimensions, Point, Primitive, RgbColor, Size};
use multiboot2::{FramebufferField};
use simple_psf::{ParseError, Psf};
use x86_64::structures::paging::PageTableFlags;

use crate::common::color::nearest_color;
use crate::common::font::FONT;
use crate::dev::lease::LeaseCell;
use crate::dev::registry::DEVICE_REGISTRY;
use crate::memory::map_ptr;

#[derive(Debug, Clone)]
pub enum PixelEncoding {
    RGB { red: FramebufferField, green: FramebufferField, blue: FramebufferField },
    Indexed { palette: Vec<Rgb888> },
}

#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    pub address: usize,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
    pub pixel_encoding: PixelEncoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramebufferError {
    NullAddress,
    ZeroSized,
    UnsupportedBpp(u8),
    PitchTooSmall { pitch: u32, required: u32 },
    EmptyPalette,
    InvalidColorField(FramebufferField),
    InvalidFont(ParseError),
}

pub struct Framebuffer {
    info: FramebufferInfo,
    bytes_per_pixel: u8,
    cache: BTreeMap<Rgb888, usize>,
    back: Vec<u8>,
    font: &'static Psf<'static>,
}

impl Framebuffer {
    pub unsafe fn new(info: &FramebufferInfo) -> Result<Self, FramebufferError> {
        if info.address == 0 {
            return Err(FramebufferError::NullAddress);
        }
        if info.width == 0 || info.height == 0 {
            return Err(FramebufferError::ZeroSized);
        }
        let font = FONT.as_ref().map_err(|err| FramebufferError::InvalidFont(*err))?;
        let bytes_per_pixel = match info.bpp {
            8 => 1,
            15 | 16 => 2,
            24 => 3,
            32 => 4,
            other => return Err(FramebufferError::UnsupportedBpp(other)),
        };
 
        let required = info.width * bytes_per_pixel as u32;
        if info.pitch < required {
            return Err(FramebufferError::PitchTooSmall {
                pitch: info.pitch,
                required,
            });
        }
 
        match &info.pixel_encoding {
            PixelEncoding::RGB { red, green, blue } => {
                for field in [red, green, blue] {
                    if field.position as u32 + field.size as u32 > 32 {
                        return Err(FramebufferError::InvalidColorField(field.clone()));
                    }
                }
            }
            PixelEncoding::Indexed { palette } => {
                if palette.is_empty() {
                    return Err(FramebufferError::EmptyPalette);
                }
            }
        }

        let framebuffer_size = info.pitch as usize * info.height as usize;

        let mapping_flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL | PageTableFlags::WRITABLE | PageTableFlags::WRITE_THROUGH;
        let mapped_address = map_ptr(info.address, framebuffer_size, mapping_flags).expect("Mapping failed failed");
        let mut info = info.clone();
        info.address = mapped_address;

        let back = alloc::vec![0u8; framebuffer_size];

        Ok(Self {
            info,
            bytes_per_pixel,
            cache: BTreeMap::new(),
            back,
            font,
        })
    }

    pub fn info(&self) -> &FramebufferInfo {
        &self.info
    }
 
    fn encode(&mut self, color: Rgb888) -> u32 {
        let (r, g, b) = (color.r(), color.g(), color.b());
 
        match &self.info.pixel_encoding {
            PixelEncoding::RGB { red, green, blue } => {
                    encode_field(red, r) | encode_field(green, g) | encode_field(blue, b)
            }
            PixelEncoding::Indexed { palette } => {
                *self.cache.entry(color).or_insert_with(|| {
                    nearest_color(palette, color)
                }) as u32
            }
        }
    }

    fn write_raw(&mut self, x: u32, y: u32, raw: u32) {
        let bpp = self.bytes_per_pixel as usize;
        let offset = y as usize * self.info.pitch as usize + x as usize * bpp;
        let bytes = raw.to_le_bytes();
        self.back[offset..offset + bpp].copy_from_slice(&bytes[..bpp]);
    }

    pub fn flush(&mut self) {
        let dst = self.info.address as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(self.back.as_ptr(), dst, self.back.len());
            core::arch::x86_64::_mm_sfence();
        }
    }
}

fn encode_field(field: &FramebufferField, value: u8) -> u32 {
    if field.size == 0 {
        return 0;
    }
 
    let scaled = if field.size <= 8 {
        (value >> (8 - field.size)) as u32
    } else {
        let extra = field.size - 8;
        let mut scaled = (value as u32) << extra;
        if extra < 8 {
            scaled |= (value as u32) >> (8 - extra);
        }
        scaled
    };
 
    scaled << field.position
}

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        Size::new(self.info.width, self.info.height)
    }
}

impl DrawTarget for Framebuffer {
    type Color = Rgb888;
    type Error = !;
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>> {
        for Pixel(point, color) in pixels {
            if point.x < 0 || point.y < 0 {
                continue;
            }
            let (x, y) = (point.x as u32, point.y as u32);
            if x >= self.info.width || y >= self.info.height {
                continue;
            }

            let raw = self.encode(color);
            self.write_raw(x, y, raw);
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box());
        if area.size.width == 0 || area.size.height == 0 {
            return Ok(());
        }

        let raw = self.encode(color);
        let bytes = raw.to_le_bytes();
        let bpp = self.bytes_per_pixel as usize;
        let pitch = self.info.pitch as usize;

        let x0 = area.top_left.x as usize;
        let y0 = area.top_left.y as usize;
        let w = area.size.width as usize;

        for y in y0..y0 + area.size.height as usize {
            let start = y * pitch + x0 * bpp;
            for px in self.back[start..start + w * bpp].chunks_exact_mut(bpp) {
                px.copy_from_slice(&bytes[..bpp]);
            }
        }
        Ok(())
    }
}

impl TextSurface for Framebuffer {
    fn width(&self) -> u32 {
        self.info.width / self.font.glyph_width as u32
    }

    fn height(&self) -> u32 {
        self.info.height / self.font.glyph_height as u32
    }

    fn put(&mut self, x: u32, y: u32, cell: super::Cell) -> Result<(), super::TextSurfaceError> {
        if x >= self.width() || y >= self.height() {
            return Err(TextSurfaceError::OutOfBounds { x, y, width: self.width(), height: self.height() });
        }
        let pixels = self.font.get_glyph_pixels(cell.c as usize)
            .ok_or(TextSurfaceError::UnsupportedChar { ch: cell.c })?;
        let fg = self.encode(cell.fg);
        let bg = self.encode(cell.bg);

        let x0 = x * self.font.glyph_width as u32;
        let y0 = y * self.font.glyph_height as u32;

        for (i, on) in pixels.enumerate() {
            let i = i as u32;
            self.write_raw(x0 + i % self.font.glyph_width as u32, y0 + i / self.font.glyph_width as u32, if on { fg } else { bg });
        }
        Ok(())
    }

    fn scroll_up(&mut self, rows: u32) {
        if rows == 0 {
            return;
        }
        if rows >= self.height() {
            self.back.fill(0);
            return;
        }

        let shift = rows as usize * self.font.glyph_height as usize * self.info.pitch as usize;
        let len = self.back.len();
        self.back.copy_within(shift..len, 0);
        self.back[len - shift..].fill(0);
    }

    fn present(&mut self) {
        self.flush();
    }
}

pub fn init(info: &FramebufferInfo) {
    let framebuffer = unsafe { Framebuffer::new(info) }.expect("Given framebuffer info has errors");
    let framebuffer_dev = Arc::new(LeaseCell::new(framebuffer));
    let info = DEVICE_REGISTRY.write().register::<Framebuffer>(framebuffer_dev.clone());
    DEVICE_REGISTRY.write().add_role::<dyn TextSurface>(info, framebuffer_dev);
}

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::Text,
};

use super::{TextSurface, TextSurfaceError};

#[allow(unused)]
pub fn test_framebuffer<D>(fb: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let Size { width: w, height: h } = fb.bounding_box().size;

    fb.clear(Rgb888::BLACK)?;

    // Border: width, height and pitch are all wrong if this doesn't close.
    fb.bounding_box()
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
        .draw(fb)?;

    // Colour bars: swapped order means your channel packing is wrong.
    let third = w / 3;
    for (i, c) in [Rgb888::RED, Rgb888::GREEN, Rgb888::BLUE].iter().enumerate() {
        Rectangle::new(
            Point::new((i as u32 * third) as i32 + 4, 4),
            Size::new(third - 8, 40),
        )
        .into_styled(PrimitiveStyle::with_fill(*c))
        .draw(fb)?;
    }

    // Diagonal: skews or wraps if pitch is wrong, even when the border looks fine.
    Line::new(Point::zero(), Point::new(w as i32 - 1, h as i32 - 1))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 1))
        .draw(fb)?;

    Text::new(
        "KalneonOS framebuffer OK",
        Point::new(12, 70),
        MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
    )
    .draw(fb)?;

    Ok(())
}
