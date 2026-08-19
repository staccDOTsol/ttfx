/// Easing function trait.
pub trait Easing {
    fn ease(&self, t: f64) -> f64;
}

/// Linear easing.
pub struct Linear;
impl Easing for Linear {
    fn ease(&self, t: f64) -> f64 {
        t
    }
}

/// Quadratic ease-in.
pub struct QuadIn;
impl Easing for QuadIn {
    fn ease(&self, t: f64) -> f64 {
        t * t
    }
}

/// Quadratic ease-out.
pub struct QuadOut;
impl Easing for QuadOut {
    fn ease(&self, t: f64) -> f64 {
        t * (2.0 - t)
    }
}

/// Quadratic ease-in-out.
pub struct QuadInOut;
impl Easing for QuadInOut {
    fn ease(&self, t: f64) -> f64 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
}

/// Cubic ease-in.
pub struct CubicIn;
impl Easing for CubicIn {
    fn ease(&self, t: f64) -> f64 {
        t * t * t
    }
}

/// Cubic ease-out.
pub struct CubicOut;
impl Easing for CubicOut {
    fn ease(&self, t: f64) -> f64 {
        let p = t - 1.0;
        p * p * p + 1.0
    }
}

/// Cubic ease-in-out.
pub struct CubicInOut;
impl Easing for CubicInOut {
    fn ease(&self, t: f64) -> f64 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            let p = t - 1.0;
            4.0 * p * p * p + 1.0
        }
    }
}

/// Sine ease-in.
pub struct SineIn;
impl Easing for SineIn {
    fn ease(&self, t: f64) -> f64 {
        1.0 - (t * std::f64::consts::PI / 2.0).cos()
    }
}

/// Sine ease-out.
pub struct SineOut;
impl Easing for SineOut {
    fn ease(&self, t: f64) -> f64 {
        (t * std::f64::consts::PI / 2.0).sin()
    }
}

/// Sine ease-in-out.
pub struct SineInOut;
impl Easing for SineInOut {
    fn ease(&self, t: f64) -> f64 {
        0.5 * (1.0 - (t * std::f64::consts::PI).cos())
    }
}

/// Get an easing function by name.
pub fn get_easing(name: &str) -> Option<Box<dyn Easing>> {
    match name {
        "linear" => Some(Box::new(Linear)),
        "quad_in" => Some(Box::new(QuadIn)),
        "quad_out" => Some(Box::new(QuadOut)),
        "quad_in_out" => Some(Box::new(QuadInOut)),
        "cubic_in" => Some(Box::new(CubicIn)),
        "cubic_out" => Some(Box::new(CubicOut)),
        "cubic_in_out" => Some(Box::new(CubicInOut)),
        "sine_in" => Some(Box::new(SineIn)),
        "sine_out" => Some(Box::new(SineOut)),
        "sine_in_out" => Some(Box::new(SineInOut)),
        _ => None,
    }
}
