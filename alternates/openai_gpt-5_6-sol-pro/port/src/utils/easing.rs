use std::f64::consts::PI;

pub type EasingFunction = fn(f64) -> f64;

fn normalized(progress: f64) -> f64 {
    progress.clamp(0.0, 1.0)
}

pub fn linear(progress: f64) -> f64 {
    normalized(progress)
}

pub fn in_sine(progress: f64) -> f64 {
    let progress = normalized(progress);
    1.0 - (progress * PI / 2.0).cos()
}

pub fn out_sine(progress: f64) -> f64 {
    let progress = normalized(progress);
    (progress * PI / 2.0).sin()
}

pub fn in_out_sine(progress: f64) -> f64 {
    let progress = normalized(progress);
    -((PI * progress).cos() - 1.0) / 2.0
}

pub fn in_quad(progress: f64) -> f64 {
    normalized(progress).powi(2)
}

pub fn out_quad(progress: f64) -> f64 {
    let progress = normalized(progress);
    1.0 - (1.0 - progress).powi(2)
}

pub fn in_out_quad(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress < 0.5 {
        2.0 * progress.powi(2)
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
    }
}

pub fn in_cubic(progress: f64) -> f64 {
    normalized(progress).powi(3)
}

pub fn out_cubic(progress: f64) -> f64 {
    let progress = normalized(progress);
    1.0 - (1.0 - progress).powi(3)
}

pub fn in_out_cubic(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress < 0.5 {
        4.0 * progress.powi(3)
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(3) / 2.0
    }
}

pub fn in_quart(progress: f64) -> f64 {
    normalized(progress).powi(4)
}

pub fn out_quart(progress: f64) -> f64 {
    let progress = normalized(progress);
    1.0 - (1.0 - progress).powi(4)
}

pub fn in_out_quart(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress < 0.5 {
        8.0 * progress.powi(4)
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(4) / 2.0
    }
}

pub fn in_quint(progress: f64) -> f64 {
    normalized(progress).powi(5)
}

pub fn out_quint(progress: f64) -> f64 {
    let progress = normalized(progress);
    1.0 - (1.0 - progress).powi(5)
}

pub fn in_out_quint(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress < 0.5 {
        16.0 * progress.powi(5)
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(5) / 2.0
    }
}

pub fn in_expo(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress == 0.0 {
        0.0
    } else {
        2.0_f64.powf(10.0 * progress - 10.0)
    }
}

pub fn out_expo(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f64.powf(-10.0 * progress)
    }
}

pub fn in_out_expo(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress == 0.0 || progress == 1.0 {
        progress
    } else if progress < 0.5 {
        2.0_f64.powf(20.0 * progress - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f64.powf(-20.0 * progress + 10.0)) / 2.0
    }
}

pub fn in_circ(progress: f64) -> f64 {
    let progress = normalized(progress);
    1.0 - (1.0 - progress.powi(2)).sqrt()
}

pub fn out_circ(progress: f64) -> f64 {
    let progress = normalized(progress);
    (1.0 - (progress - 1.0).powi(2)).sqrt()
}

pub fn in_out_circ(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress < 0.5 {
        (1.0 - (1.0 - (2.0 * progress).powi(2)).sqrt()) / 2.0
    } else {
        ((1.0 - (-2.0 * progress + 2.0).powi(2)).sqrt() + 1.0) / 2.0
    }
}

pub fn in_back(progress: f64) -> f64 {
    let progress = normalized(progress);
    const C1: f64 = 1.70158;
    const C3: f64 = C1 + 1.0;
    C3 * progress.powi(3) - C1 * progress.powi(2)
}

pub fn out_back(progress: f64) -> f64 {
    let progress = normalized(progress);
    const C1: f64 = 1.70158;
    const C3: f64 = C1 + 1.0;
    1.0 + C3 * (progress - 1.0).powi(3)
        + C1 * (progress - 1.0).powi(2)
}

pub fn out_bounce(progress: f64) -> f64 {
    let mut progress = normalized(progress);
    const N1: f64 = 7.5625;
    const D1: f64 = 2.75;

    if progress < 1.0 / D1 {
        N1 * progress * progress
    } else if progress < 2.0 / D1 {
        progress -= 1.5 / D1;
        N1 * progress * progress + 0.75
    } else if progress < 2.5 / D1 {
        progress -= 2.25 / D1;
        N1 * progress * progress + 0.9375
    } else {
        progress -= 2.625 / D1;
        N1 * progress * progress + 0.984375
    }
}

pub fn in_bounce(progress: f64) -> f64 {
    1.0 - out_bounce(1.0 - normalized(progress))
}

pub fn in_out_bounce(progress: f64) -> f64 {
    let progress = normalized(progress);

    if progress < 0.5 {
        (1.0 - out_bounce(1.0 - 2.0 * progress)) / 2.0
    } else {
        (1.0 + out_bounce(2.0 * progress - 1.0)) / 2.0
    }
}

pub fn make_easing(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) -> impl Fn(f64) -> f64 {
    move |progress| {
        let target_x = normalized(progress);
        let mut low = 0.0;
        let mut high = 1.0;
        let mut t = target_x;

        for _ in 0..20 {
            t = (low + high) / 2.0;
            let x = cubic_component(t, x1, x2);

            if x < target_x {
                low = t;
            } else {
                high = t;
            }
        }

        cubic_component(t, y1, y2)
    }
}

fn cubic_component(t: f64, control_1: f64, control_2: f64) -> f64 {
    let inverse = 1.0 - t;

    3.0 * inverse.powi(2) * t * control_1
        + 3.0 * inverse * t.powi(2) * control_2
        + t.powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_endpoints_are_stable() {
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(1.0), 1.0);
        assert_eq!(in_out_sine(0.0), 0.0);
        assert_eq!(in_out_sine(1.0), 1.0);
    }
}
