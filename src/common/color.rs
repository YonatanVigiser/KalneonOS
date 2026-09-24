use embedded_graphics::prelude::RgbColor;

/// Redmean approximation
pub fn color_distance(a: impl RgbColor, b: impl RgbColor) -> u32 {
    let (ar, ag, ab) = (a.r() as i32, a.g() as i32, a.b() as i32);
    let (br, bg, bb) = (b.r() as i32, b.g() as i32, b.b() as i32);

    let rmean = (ar + br) / 2;
    let (dr, dg, db) = (ar - br, ag - bg, ab - bb);

    ((((512 + rmean) * dr * dr) >> 8) + (4 * dg * dg) + (((767 - rmean) * db * db) >> 8)) as u32
}

pub fn nearest_color(palette: &[impl RgbColor], target: impl RgbColor) -> usize {
    palette
        .iter()
        .enumerate()
        .min_by_key(|(_, c)| color_distance(**c, target))
        .map(|(i, _)| i)
        .expect("empty palette")
}
