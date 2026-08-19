//! Colors, color pairs, and gradients (mirrors terminaltexteffects.utils.graphics).

/// A 24-bit RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex color string like `"ff00aa"` or `"#ff00aa"`.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Linear interpolation between two colors.
    pub fn lerp(a: Color, b: Color, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        let mix = |x: u8, y: u8| -> u8 {
            (x as f64 + (y as f64 - x as f64) * t).round().clamp(0.0, 255.0) as u8
        };
        Color::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
    }
}

/// Foreground/background color pair; either side may be unset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ColorPair {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl ColorPair {
    pub fn new(fg: Option<Color>, bg: Option<Color>) -> Self {
        Self { fg, bg }
    }

    pub fn fg_only(fg: Color) -> Self {
        Self { fg: Some(fg), bg: None }
    }
}

/// A sequence of colors interpolated between gradient stops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gradient {
    pub spectrum: Vec<Color>,
}

impl Gradient {
    /// Build a gradient with `steps` interpolated colors between each pair of stops.
    pub fn new(stops: &[Color], steps: usize) -> Self {
        let mut spectrum = Vec::new();
        match stops.len() {
            0 => {}
            1 => spectrum.push(stops[0]),
            _ => {
                let steps = steps.max(1);
                for pair in stops.windows(2) {
                    for i in 0..steps {
                        let t = i as f64 / steps as f64;
                        spectrum.push(Color::lerp(pair[0], pair[1], t));
                    }
                }
                spectrum.push(*stops.last().expect("non-empty stops"));
            }
        }
        Self { spectrum }
    }

    /// Color at fractional progress `t` (0..=1) through the spectrum.
    pub fn get_color_at_fraction(&self, t: f64) -> Option<Color> {
        if self.spectrum.is_empty() {
            return None;
        }
        let t = t.clamp(0.0, 1.0);
        let idx = ((self.spectrum.len() - 1) as f64 * t).round() as usize;
        self.spectrum.get(idx).copied()
    }
}
