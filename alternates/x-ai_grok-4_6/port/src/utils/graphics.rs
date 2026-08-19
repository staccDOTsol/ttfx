#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('#');
        if s.len() != 6 {
            return None;
        }
        let n = u32::from_str_radix(s, 16).ok()?;
        Some(Self {
            r: ((n >> 16) & 0xff) as u8,
            g: ((n >> 8) & 0xff) as u8,
            b: (n & 0xff) as u8,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ColorPair {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

#[derive(Clone, Debug)]
pub struct Gradient {
    pub stops: Vec<Color>,
    pub steps: usize,
}

impl Gradient {
    pub fn new(stops: Vec<Color>, steps: usize) -> Self {
        Self {
            stops,
            steps: steps.max(1),
        }
    }

    pub fn colors(&self) -> Vec<Color> {
        if self.stops.is_empty() {
            return Vec::new();
        }
        if self.stops.len() == 1 || self.steps <= 1 {
            return vec![self.stops[0]];
        }
        let mut out = Vec::with_capacity(self.steps);
        let segs = self.stops.len() - 1;
        for i in 0..self.steps {
            let t = i as f64 / (self.steps - 1) as f64;
            let f = t * segs as f64;
            let idx = (f.floor() as usize).min(segs - 1);
            let local = f - idx as f64;
            let a = self.stops[idx];
            let b = self.stops[idx + 1];
            out.push(Color {
                r: lerp_u8(a.r, b.r, local),
                g: lerp_u8(a.g, b.g, local),
                b: lerp_u8(a.b, b.b, local),
            });
        }
        out
    }
}

fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t).round() as u8
}
