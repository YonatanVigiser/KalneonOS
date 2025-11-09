//#[link_section = ".multiboot"]

const MULTIBOOT2_MAGIC : u32 = 0xE85250D6;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const ARCHITECTURE: u32 = 0;

const TAGS: [Multiboot2Tag; 1] = [
    Multiboot2Tag::new(0, false, &[]),
];

const HEADER_LENGTH: u32;
const CHECKSUM: u32 = 0u32.wrapping_sub(MULTIBOOT2_MAGIC + ARCHITECTURE + HEADER_LENGTH);


struct Multiboot2Tag {
    tag_type: u16,
    flags: u16,
    size: u32,
    tag_info: &'static [u32],
}

impl Multiboot2Tag {
    const fn new(tag_type: u16, optional: bool, tag_info: &'static [u32]) -> Self {
        Multiboot2Tag {
            tag_type,
            flags: optional as u16,
            size: 8 + tag_info.len() as u32,
            tag_info,
        }
    }
}

struct Multiboot2Header {
    magic: u32,
    arch: u32,
    header_length: u32,
    checksum: u32,
    tags: &'static [Multiboot2Tag]
}
