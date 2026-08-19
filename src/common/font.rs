use simple_psf::{ParseError, Psf};

static FONT_DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/terminus16-uni.psf"));
pub static FONT: Result<Psf, ParseError> = Psf::parse(FONT_DATA);

