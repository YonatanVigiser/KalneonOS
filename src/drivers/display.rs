use multiboot2::{FramebufferTag, FramebufferType};

use self::framebuffer::{FramebufferInfo, PixelEncoding};

pub mod framebuffer;
pub mod vga;

pub enum DisplayInfo {
    Graphics {
        info: FramebufferInfo,
    },
    Text {
        address: usize,
        cols: u32,
        rows: u32,
        pitch: u32,
        bpp: u8,
    },
}

impl From<&FramebufferTag> for DisplayInfo {
    fn from(value: &FramebufferTag) -> Self {
        match value.buffer_type().unwrap() {
            FramebufferType::RGB { red, green, blue } => Self::Graphics { info: FramebufferInfo {
                address: value.address() as usize,
                width: value.width(),
                heigth: value.height(),
                pitch: value.pitch(),
                bpp: value.bpp(),
                pixel_econding: PixelEncoding::RGB { red, green, blue },
            }},
            FramebufferType::Indexed { palette } => Self::Graphics { info: FramebufferInfo {
                address: value.address() as usize,
                width: value.width(),
                heigth: value.height(),
                pitch: value.pitch(),
                bpp: value.bpp(),
                pixel_econding: PixelEncoding::Indexed { palette: palette.iter().copied().collect() },
            }},
            FramebufferType::Text => Self::Text { address: value.address() as usize, cols: value.width(), rows: value.height(), pitch: value.pitch(), bpp: value.bpp() },
        }
    }
}
