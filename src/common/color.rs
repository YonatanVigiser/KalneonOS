#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8); // RGB

impl Color {
    pub const BLACK: Color = Color(0x00, 0x00, 0x00);
    pub const WHITE: Color = Color(0xff, 0xff, 0xff);
    pub const RED:   Color = Color(0xff, 0x00, 0x00);
    pub const GREEN: Color = Color(0x00, 0xff, 0x00);
    pub const BLUE:  Color = Color(0x00, 0x00, 0xff);

    pub fn inverse(&self) -> Self {
        Self(!self.0, !self.1, !self.2)
    }

    pub fn distance_to(&self, color: &Self) -> u16 {
        let r_dis = self.0.abs_diff(color.0) as u16;
        let g_dis = self.1.abs_diff(color.1) as u16;
        let b_dis = self.2.abs_diff(color.2) as u16;
        r_dis + g_dis + b_dis
    }
}

pub struct ColorPalette<'a>(pub &'a [Color]);

impl<'a> ColorPalette<'_> {
    pub fn select_closest(&self, target: &Color) -> &Color {
        let mut closest = &self.0[0];
        for color in self.0 {
            if color.distance_to(target) < closest.distance_to(target) {
                closest = color;
            }
        }
        &closest
    }

    pub fn get_closest_index(&self, target: &Color) -> usize {
        let mut closest_index = 0;
        for color_index in 1..self.0.len() {
            if self.0[color_index].distance_to(target) < self.0[closest_index].distance_to(target) {
                closest_index = color_index;
            }
        }
        closest_index
    }

    pub fn select_nth(&self, index: usize) -> &Color {
        &self.0[index]
    }
}
