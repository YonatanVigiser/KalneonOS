#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8); // RGB

impl Color {
    pub fn inverse(&self) -> Self {
        Self(!self.0, !self.1, !self.2)
    }

    pub fn distance_to(&self, color: &Self) -> u16 {
        let r_dis = self.0.abs_diff(color.0) as u16;
        let g_dis = self.1.abs_diff(color.1) as u16;
        let b_dis = self.2.abs_diff(color.2) as u16;
        r_dis + g_dis + b_dis
    }

    pub const fn red() -> Self {
        Self(255, 0, 0)
    }

    pub const fn green() -> Self {
        Self(0, 255, 0)
    }

    pub const fn blue() -> Self {
        Self(0, 0, 255)
    }

    pub const fn black() -> Self {
        Self(0, 0, 0)
    }

    pub const fn white() -> Self {
        Self(255, 255, 255)
    }
}

pub struct ColorPalette<'a>(pub &'a [Color]);

impl<'a> ColorPalette<'_> {
    pub fn select_closest(&self, target: &Color) -> &Color {
        let closest = self.0[0];
        for color in self.0 {
            if color.distance_to(target) < closest.distance_to(target) {
                closest = color;
            }
        }
        &closest
    }

    pub fn get_closest_index(&self, target: &Color) -> usize {
        let closest_index = 0;
        for color_index in 1..self.0.len {
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
