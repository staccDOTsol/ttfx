use std::f64::consts::TAU;

use super::Effect;
use crate::engine::canvas::Canvas;
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, Gradient, Style};

const LAUNCH_FRAMES: usize = 14;
const EXPLODE_FRAMES: usize = 12;
const SETTLE_FRAMES: usize = 18;
const FINAL_HOLD_FRAMES: usize = 8;
const SHELL_SIZE: usize = 8;
const SHELL_DELAY: usize = 5;

pub struct Fireworks;

impl Fireworks {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Fireworks {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Fireworks {
    fn name(&self) -> &str {
        "fireworks"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let parsed = ParsedInput::new(input);
        let final_colors = Gradient::new(
            [
                Color::new(255, 95, 109),
                Color::new(255, 195, 113),
                Color::new(111, 255, 233),
                Color::new(112, 161, 255),
                Color::new(210, 125, 255),
            ],
            parsed.height,
        )
        .colors();

        let mut rng = SmallRng::new(hash_input(input));
        let mut glyphs = parsed.glyphs;
        shuffle(&mut glyphs, &mut rng);

        let palette = [
            Color::new(255, 45, 85),
            Color::new(255, 139, 41),
            Color::new(255, 231, 76),
            Color::new(62, 255, 139),
            Color::new(48, 214, 255),
            Color::new(83, 109, 254),
            Color::new(191, 90, 242),
            Color::new(255, 55, 199),
        ];

        let mut shells = Vec::new();

        for (shell_index, chunk) in glyphs.chunks(SHELL_SIZE).enumerate() {
            let center_column = if chunk.is_empty() {
                parsed.width as i32 / 2
            } else {
                py_round(
                    chunk.iter().map(|glyph| glyph.coord.column).sum::<i32>()
                        as f64
                        / chunk.len() as f64,
                ) as i32
            };

            let center_row = if chunk.is_empty() {
                parsed.height as i32 / 3
            } else {
                py_round(
                    chunk.iter().map(|glyph| glyph.coord.row).sum::<i32>()
                        as f64
                        / chunk.len() as f64,
                ) as i32
            };

            let horizontal_jitter = rng.range_i32(-2, 3);
            let apex_column = (center_column + horizontal_jitter)
                .clamp(0, parsed.width.saturating_sub(1) as i32);
            let apex_row_limit = parsed.height.saturating_sub(1) as i32;
            let apex_row = center_row
                .min((parsed.height as i32 / 3).max(0))
                .clamp(0, apex_row_limit);
            let apex = Coord::new(apex_column, apex_row);
            let launch = Coord::new(
                apex_column,
                parsed.height.saturating_sub(1) as i32,
            );

            let color = palette[rng.range_usize(palette.len())];
            let mut particles = Vec::with_capacity(chunk.len());

            for (particle_index, glyph) in chunk.iter().enumerate() {
                let base_angle =
                    particle_index as f64 / chunk.len().max(1) as f64 * TAU;
                let angle = base_angle + rng.range_f64(-0.28, 0.28);
                let radius = rng.range_f64(2.0, 5.5);
                let burst = Coord::new(
                    apex.column + py_round(angle.cos() * radius) as i32,
                    apex.row + py_round(angle.sin() * radius * 0.55) as i32,
                );

                particles.push(Particle {
                    symbol: glyph.symbol.clone(),
                    target: glyph.coord,
                    burst,
                    final_color: final_colors
                        .get(glyph.coord.row.max(0) as usize)
                        .copied()
                        .unwrap_or(color),
                });
            }

            shells.push(Shell {
                delay: shell_index * SHELL_DELAY,
                launch,
                apex,
                color,
                particles,
            });
        }

        if shells.is_empty() {
            return blank_firework_frames(
                parsed.width,
                parsed.height,
                palette[0],
            );
        }

        let shell_duration =
            LAUNCH_FRAMES + EXPLODE_FRAMES + SETTLE_FRAMES;
        let total_frames = shells
            .last()
            .map(|shell| shell.delay + shell_duration + FINAL_HOLD_FRAMES)
            .unwrap_or(FINAL_HOLD_FRAMES);

        let mut frames = Vec::with_capacity(total_frames);

        for frame_index in 0..total_frames {
            let mut canvas = Canvas::new(parsed.width, parsed.height);
            let mut painted = false;

            for shell in &shells {
                if frame_index < shell.delay {
                    continue;
                }

                let local_frame = frame_index - shell.delay;

                if local_frame < LAUNCH_FRAMES {
                    let denominator = LAUNCH_FRAMES.saturating_sub(1).max(1);
                    let progress =
                        local_frame as f64 / denominator as f64;
                    let progress = easing::out_quart(progress);
                    let rocket = interpolate_coord(
                        shell.launch,
                        shell.apex,
                        progress,
                    );

                    let trail_progress = (progress - 0.08).max(0.0);
                    let trail = interpolate_coord(
                        shell.launch,
                        shell.apex,
                        trail_progress,
                    );

                    let trail_style = Style {
                        foreground: Some(
                            shell
                                .color
                                .lerp(Color::new(255, 255, 255), 0.35),
                        ),
                        ..Style::default()
                    };
                    let rocket_style = Style {
                        foreground: Some(shell.color),
                        bold: true,
                        ..Style::default()
                    };

                    painted |= canvas.set(trail, "·", trail_style);
                    painted |= canvas.set(rocket, "▄", rocket_style);
                    continue;
                }

                let explosion_frame = local_frame - LAUNCH_FRAMES;

                if explosion_frame < EXPLODE_FRAMES {
                    let denominator =
                        EXPLODE_FRAMES.saturating_sub(1).max(1);
                    let progress =
                        explosion_frame as f64 / denominator as f64;
                    let movement = easing::out_circ(progress);
                    let brightness =
                        1.0 - (2.0 * progress - 1.0).abs() * 0.55;
                    let explosion_color = shell.color.lerp(
                        Color::new(255, 255, 255),
                        brightness.clamp(0.0, 1.0),
                    );
                    let style = Style {
                        foreground: Some(explosion_color),
                        bold: true,
                        ..Style::default()
                    };

                    for particle in &shell.particles {
                        let position = interpolate_coord(
                            shell.apex,
                            particle.burst,
                            movement,
                        );
                        painted |= canvas.set(position, "*", style);
                    }

                    continue;
                }

                let settle_frame = explosion_frame - EXPLODE_FRAMES;

                if settle_frame < SETTLE_FRAMES {
                    let denominator =
                        SETTLE_FRAMES.saturating_sub(1).max(1);
                    let progress =
                        settle_frame as f64 / denominator as f64;
                    let movement = easing::in_out_cubic(progress);

                    for particle in &shell.particles {
                        let position = interpolate_coord(
                            particle.burst,
                            particle.target,
                            movement,
                        );
                        let color =
                            shell.color.lerp(particle.final_color, progress);
                        let style = Style {
                            foreground: Some(color),
                            bold: progress < 0.55,
                            ..Style::default()
                        };
                        let symbol = if progress < 0.45 {
                            "*"
                        } else {
                            particle.symbol.as_str()
                        };

                        painted |= canvas.set(position, symbol, style);
                    }

                    continue;
                }

                for particle in &shell.particles {
                    let style = Style {
                        foreground: Some(particle.final_color),
                        ..Style::default()
                    };
                    painted |=
                        canvas.set(particle.target, &particle.symbol, style);
                }
            }

            if !painted {
                canvas.set(
                    Coord::new(0, 0),
                    " ",
                    Style {
                        foreground: Some(Color::new(255, 255, 255)),
                        ..Style::default()
                    },
                );
            }

            frames.push(canvas.render());
        }

        frames
    }
}

#[derive(Clone)]
struct Glyph {
    symbol: String,
    coord: Coord,
}

struct Particle {
    symbol: String,
    target: Coord,
    burst: Coord,
    final_color: Color,
}

struct Shell {
    delay: usize,
    launch: Coord,
    apex: Coord,
    color: Color,
    particles: Vec<Particle>,
}

struct ParsedInput {
    width: usize,
    height: usize,
    glyphs: Vec<Glyph>,
}

impl ParsedInput {
    fn new(input: &str) -> Self {
        let input = input.strip_suffix('\n').unwrap_or(input);
        let mut lines: Vec<&str> = input.split('\n').collect();

        if lines.is_empty() {
            lines.push("");
        }

        let width = lines
            .iter()
            .map(|line| line.trim_end_matches('\r').chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);
        let mut glyphs = Vec::new();

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in
                line.trim_end_matches('\r').chars().enumerate()
            {
                if symbol.is_whitespace() {
                    continue;
                }

                glyphs.push(Glyph {
                    symbol: symbol.to_string(),
                    coord: Coord::new(column as i32, row as i32),
                });
            }
        }

        Self {
            width,
            height,
            glyphs,
        }
    }
}

fn interpolate_coord(start: Coord, end: Coord, progress: f64) -> Coord {
    let progress = progress.clamp(0.0, 1.0);
    let column = start.column as f64
        + (end.column - start.column) as f64 * progress;
    let row =
        start.row as f64 + (end.row - start.row) as f64 * progress;

    Coord::new(py_round(column) as i32, py_round(row) as i32)
}

fn py_round(value: f64) -> i64 {
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

fn blank_firework_frames(
    width: usize,
    height: usize,
    color: Color,
) -> Vec<String> {
    let mut frames = Vec::with_capacity(16);
    let center = Coord::new(
        width.saturating_sub(1) as i32 / 2,
        height.saturating_sub(1) as i32 / 2,
    );

    for frame_index in 0..16 {
        let mut canvas = Canvas::new(width, height);
        let progress = frame_index as f64 / 15.0;
        let symbol = if frame_index < 7 {
            "▄"
        } else if frame_index < 14 {
            "*"
        } else {
            " "
        };
        let style = Style {
            foreground: Some(
                color.lerp(Color::new(255, 255, 255), progress),
            ),
            bold: frame_index < 14,
            ..Style::default()
        };

        canvas.set(center, symbol, style);
        frames.push(canvas.render());
    }

    frames
}

fn hash_input(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;

    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    if hash == 0 {
        0x9e3779b97f4a7c15
    } else {
        hash
    }
}

fn shuffle<T>(values: &mut [T], rng: &mut SmallRng) {
    for index in (1..values.len()).rev() {
        let other = rng.range_usize(index + 1);
        values.swap(index, other);
    }
}

struct SmallRng {
    state: u64,
}

impl SmallRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e3779b97f4a7c15
            } else {
                seed
            },
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

    fn unit_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1_u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    fn range_f64(&mut self, start: f64, end: f64) -> f64 {
        start + (end - start) * self.unit_f64()
    }

    fn range_usize(&mut self, end: usize) -> usize {
        if end <= 1 {
            0
        } else {
            (self.next_u64() % end as u64) as usize
        }
    }

    fn range_i32(&mut self, start: i32, end: i32) -> i32 {
        if end <= start {
            start
        } else {
            start + self.range_usize((end - start) as usize) as i32
        }
    }
}
