use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::RgbColor;
use multiboot2::{FramebufferColor, FramebufferField};

pub enum PixelEncoding {
    RGB { red: FramebufferField, green: FramebufferField, blue: FramebufferField },
    Indexed { palette: Vec<FramebufferColor> },
}

pub struct FramebufferInfo {
    pub address: usize,
    pub width: u32,
    pub heigth: u32,
    pub pitch: u32,
    pub bpp: u8,
    pub pixel_econding: PixelEncoding,
}

fn nearest_palette_entry(palette: &[FramebufferColor], r: u8, g: u8, b: u8) -> usize {
    let mut best = 0;
    let mut best_dist = i32::MAX;
 
    // Indices are stored as u16; a longer palette cannot be addressed anyway.
    for (i, entry) in palette.iter().enumerate().take(u16::MAX as usize) {
        let dr = entry.red as i32 - r as i32;
        let dg = entry.green as i32 - g as i32;
        let db = entry.blue as i32 - b as i32;
        let dist = dr * dr + dg * dg + db * db;
        if dist < best_dist {
            best_dist = dist;
            best = i;
            if dist == 0 {
                break;
            }
        }
    }
 
    best
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramebufferError {
    NullAddress,
    ZeroSized,
    UnsupportedBpp(u8),
    PitchTooSmall { pitch: u32, required: u32 },
    EmptyPalette,
    InvalidColorField(FramebufferField),
}

pub struct Framebuffer {
    info: FramebufferInfo,
    bytes_per_pixel: u8,
    cache: BTreeMap<Rgb888, usize>,
    back: Vec<u8>,
}

unsafe impl Send for Framebuffer {}

impl Framebuffer {
    pub unsafe fn new(mut info: FramebufferInfo) -> Result<Self, FramebufferError> {
        if info.address == 0 {
            return Err(FramebufferError::NullAddress);
        }
        if info.width == 0 || info.heigth == 0 {
            return Err(FramebufferError::ZeroSized);
        }
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
 
        match &mut info.pixel_econding {
            PixelEncoding::RGB { red, green, blue } => {
                for field in [red, green, blue] {
                    if field.position + field.size > 32 {
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
 
        let back = alloc::vec![0u8; info.pitch as usize * info.heigth as usize];
 
        Ok(Self {
            info,
            bytes_per_pixel,
            cache: BTreeMap::new(),
            back,
        })
    }

        pub fn width(&self) -> u32 {
        self.info.width
    }
 
    pub fn height(&self) -> u32 {
        self.info.heigth
    }
 
    pub fn pitch(&self) -> usize {
        self.info.pitch as usize
    }
 
    pub fn bpp(&self) -> u8 {
        self.info.bpp
    }
 
    fn encode(&mut self, color: Rgb888) -> u32 {
        let (r, g, b) = (color.r(), color.g(), color.b());
 
        match &self.info.pixel_econding {
            PixelEncoding::RGB { red, green, blue } => {
                    encode_field(red, r) | encode_field(green, g) | encode_field(blue, b)
            }
            PixelEncoding::Indexed { palette } => {
                *self.cache.entry(color).or_insert_with(|| {
                    nearest_palette_entry(palette, r, g, b)
                }) as u32
            }
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
