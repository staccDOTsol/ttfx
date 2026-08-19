/// RGB color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

/// A pair of foreground and background colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

impl ColorPair {
    pub fn new(fg: Color, bg: Color) -> Self {
        ColorPair { fg, bg }
    }
}

/// A gradient defined by color stops.
#[derive(Clone, Debug)]
pub struct Gradient {
    pub stops: Vec<(f64, Color)>, // position (0..1), color
}

impl Gradient {
    pub fn new(stops: Vec<(f64, Color)>) -> Self {
        Gradient { stops }
    }

    pub fn color_at(&self, t: f64) -> Color {
        if self.stops.is_empty() {
            return Color::BLACK;
        }
        if t <= self.stops[0].0 {
            return self.stops[0].1;
        }
        if t >= self.stops.last().unwrap().0 {
            return self.stops.last().unwrap().1;
        }
        for i in 0..self.stops.len() - 1 {
            let (t0, c0) = self.stops[i];
            let (t1, c1) = self.stops[i + 1];
            if t >= t0 && t <= t1 {
                let frac = (t - t0) / (t1 - t0);
                let r = (c0.r as f64 + (c1.r as f64 - c0.r as f64) * frac).round() as u8;
                let g = (c0.g as f64 + (c1.g as f64 - c0.g as f64) * frac).round() as u8;
                let b = (c0.b as f64 + (c1.b as f64 - c0.b as f64) * frac).round() as u8;
                return Color::new(r, g, b);
            }
        }
        Color::BLACK
    }
}
