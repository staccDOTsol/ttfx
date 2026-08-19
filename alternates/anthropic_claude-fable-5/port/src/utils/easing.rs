//! Easing functions (subset of terminaltexteffects.utils.easing).

use std::f64::consts::PI;

/// An easing function maps progress `t` in 0..=1 to eased progress.
pub type EasingFunction = fn(f64) -> f64;

pub fn linear(t: f64) -> f64 {
    t
}

pub fn in_sine(t: f64) -> f64 {
    1.0 - ((t * PI) / 2.0).cos()
}

pub fn out_sine(t: f64) -> f64 {
    ((t * PI) / 2.0).sin()
}

pub fn in_out_sine(t: f64) -> f64 {
    -((PI * t).cos() - 1.0) / 2.0
}

pub fn in_quad(t: f64) -> f64 {
    t * t
}

pub fn out_quad(t: f64) -> f64 {
    1.0 - (1.0 - t) * (1.0 - t)
}

pub fn in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

pub fn in_cubic(t: f64) -> f64 {
    t * t * t
}

pub fn out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

pub fn in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

pub fn in_expo(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else {
        (2.0f64).powf(10.0 * t - 10.0)
    }
}

pub fn out_expo(t: f64) -> f64 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - (2.0f64).powf(-10.0 * t)
    }
}

pub fn out_bounce(t: f64) -> f64 {
    const N1: f64 = 7.5625;
    const D1: f64 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}
