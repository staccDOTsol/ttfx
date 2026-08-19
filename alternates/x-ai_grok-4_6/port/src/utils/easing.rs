#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InSine,
    OutSine,
    InOutSine,
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}

impl Easing {
    pub fn apply(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::InQuad => t * t,
            Self::OutQuad => t * (2.0 - t),
            Self::InOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Self::InCubic => t * t * t,
            Self::OutCubic => {
                let p = t - 1.0;
                p * p * p + 1.0
            }
            Self::InOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let p = 2.0 * t - 2.0;
                    0.5 * p * p * p + 1.0
                }
            }
            Self::InSine => 1.0 - (t * std::f64::consts::FRAC_PI_2).cos(),
            Self::OutSine => (t * std::f64::consts::FRAC_PI_2).sin(),
            Self::InOutSine => 0.5 * (1.0 - (std::f64::consts::PI * t).cos()),
        }
    }
}

pub fn make_easing(x1: f64, y1: f64, x2: f64, y2: f64) -> impl Fn(f64) -> f64 {
    move |t: f64| {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let _cx = 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t;
        3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t
    }
}
