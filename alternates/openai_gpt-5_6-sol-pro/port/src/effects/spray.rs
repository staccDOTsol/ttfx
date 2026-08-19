
use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::graphics::{Color, Gradient, Style};
use crate::utils::Coord;

pub struct Spray;

impl Spray {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Spray {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Spray {
    fn name(&self) -> &str {
        "spray"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines = input_lines(input);
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let gradient = Gradient::new(
            [
                Color::new(0x8a, 0x00, 0x8a),
                Color::new(0x00, 0xd1, 0xff),
                Color::new(0xff, 0xff, 0xff),
            ],
            12,
        );
        let colors = gradient.colors();
        let origin = Coord::new(
            ((width - 1) / 2) as i32,
            ((height - 1) / 2) as i32,
        );

        let mut random = SprayRng::new(hash_input(input));
        let mut particles = Vec::new();

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let target = Coord::new(column as i32, row as i32);
                let gradient_row = height.saturating_sub(row + 1);
                let color_index = if height <= 1 || colors.len() <= 1 {
                    0
                } else {
                    gradient_row * (colors.len() - 1) / (height - 1)
                };

                let dx = (target.column - origin.column) as f64;
                let dy = (target.row - origin.row) as f64 * 2.0;
                let distance = dx.hypot(dy);
                let speed = random.range_f64(0.4, 1.0);
                let duration = round_half_even(distance / speed).max(1) as u32;

                particles.push(Particle {
                    symbol: symbol.to_string(),
                    origin,
                    target,
                    position: origin,
                    style: Style {
                        foreground: Some(colors[color_index]),
                        ..Style::default()
                    },
                    duration,
                    elapsed: 0,
                    launched: false,
                    complete: false,
                });
            }
        }

        if particles.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.set(
                origin,
                " ",
                Style {
                    foreground: Some(colors.last().copied().unwrap_or(Color::new(
                        0xff, 0xff, 0xff,
                    ))),
                    ..Style::default()
                },
            );
            return vec![canvas.render()];
        }

        let mut pending: Vec<usize> = (0..particles.len()).collect();
        random.shuffle(&mut pending);

        let spray_volume = ((particles.len() as f64 * 0.005) as usize).max(1);
        let mut frames = Vec::new();

        while !pending.is_empty() || particles.iter().any(|particle| !particle.complete) {
            for _ in 0..spray_volume {
                let Some(index) = pending.pop() else {
                    break;
                };
                particles[index].launched = true;
            }

            for particle in &mut particles {
                if !particle.launched || particle.complete {
                    continue;
                }

                particle.elapsed = particle.elapsed.saturating_add(1);
                let progress =
                    (particle.elapsed as f64 / particle.duration as f64).min(1.0);
                let eased_progress = easing::out_expo(progress);

                particle.position = interpolate(
                    particle.origin,
                    particle.target,
                    eased_progress,
                );

                if particle.elapsed >= particle.duration {
                    particle.position = particle.target;
                    particle.complete = true;
                }
            }

            let mut canvas = Canvas::new(width, height);
            for particle in &particles {
                if particle.launched {
                    canvas.set(
                        particle.position,
                        particle.symbol.clone(),
                        particle.style,
                    );
                }
            }
            frames.push(canvas.render());
        }

        frames
    }
}

struct Particle {
    symbol: String,
    origin: Coord,
    target: Coord,
    position: Coord,
    style: Style,
    duration: u32,
    elapsed: u32,
    launched: bool,
    complete: bool,
}

fn input_lines(input: &str) -> Vec<String> {
    if input.is_empty() {
        return Vec::new();
    }

    let mut lines = input
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_owned())
        .collect::<Vec<_>>();

    if input.ends_with('\n') {
        lines.pop();
    }

    lines
}

fn interpolate(start: Coord, end: Coord, progress: f64) -> Coord {
    let progress = progress.clamp(0.0, 1.0);
    let column =
        start.column as f64 + (end.column - start.column) as f64 * progress;
    let row = start.row as f64 + (end.row - start.row) as f64 * progress;

    Coord::new(
        round_half_even(column) as i32,
        round_half_even(row) as i32,
    )
}

fn round_half_even(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }

    let floor = value.floor();
    let fraction = value - floor;

    if fraction < 0.5 {
        floor as i64
    } else if fraction > 0.5 {
        floor as i64 + 1
    } else {
        let lower = floor as i64;
        if lower % 2 == 0 {
            lower
        } else {
            lower + 1
        }
    }
}

fn hash_input(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    if hash == 0 {
        0x9e37_79b9_7f4a_7c15
    } else {
        hash
    }
}

struct SprayRng {
    state: u64,
}

impl SprayRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.max(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn range_f64(&mut self, minimum: f64, maximum: f64) -> f64 {
        let unit = (self.next_u64() >> 11) as f64
            / ((1_u64 << 53) - 1) as f64;
        minimum + (maximum - minimum) * unit
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let selected = (self.next_u64() % (index as u64 + 1)) as usize;
            values.swap(index, selected);
        }
    }
}
