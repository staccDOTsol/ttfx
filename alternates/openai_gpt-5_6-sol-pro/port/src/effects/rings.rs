
use std::f64::consts::TAU;

use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::{Color, Coord, Gradient, Style};

#[derive(Clone, Debug)]
struct Glyph {
    symbol: String,
    input_coord: Coord,
    final_color: Color,
}

#[derive(Clone, Copy, Debug)]
struct RingPosition {
    ring: usize,
    angle: f64,
    radius_ratio: f64,
}

pub struct Rings;

impl Rings {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Rings {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Rings {
    fn name(&self) -> &str {
        "rings"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut lines = input
            .split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .collect::<Vec<_>>();

        if input.ends_with('\n') && lines.last().is_some_and(|line| line.is_empty()) {
            lines.pop();
        }
        if lines.is_empty() {
            lines.push("");
        }

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let final_gradient = Gradient::new(
            [
                Color::new(0xab, 0x48, 0xff),
                Color::new(0xe7, 0xb2, 0xff),
                Color::new(0xff, 0xfe, 0xbd),
            ],
            width,
        )
        .colors();

        let mut glyphs = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                glyphs.push(Glyph {
                    symbol: symbol.to_string(),
                    input_coord: Coord::new(column as i32, row as i32),
                    final_color: final_gradient[column.min(final_gradient.len() - 1)],
                });
            }
        }

        if glyphs.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style {
                    foreground: Some(Color::new(0xab, 0x48, 0xff)),
                    ..Style::default()
                },
            );
            return vec![canvas.render()];
        }

        let mut ring_positions = Vec::with_capacity(glyphs.len());
        let mut consumed = 0;
        let mut ring = 0;

        while consumed < glyphs.len() {
            let capacity = 6 * (ring + 1);
            let count = capacity.min(glyphs.len() - consumed);

            for offset in 0..count {
                ring_positions.push(RingPosition {
                    ring,
                    angle: TAU * offset as f64 / count as f64,
                    radius_ratio: 0.0,
                });
            }

            consumed += count;
            ring += 1;
        }

        let ring_count = ring.max(1);
        for position in &mut ring_positions {
            position.radius_ratio = (position.ring + 1) as f64 / ring_count as f64;
        }

        let ring_colors = Gradient::new(
            [
                Color::new(0xff, 0xfe, 0xbd),
                Color::new(0xe7, 0xb2, 0xff),
                Color::new(0xab, 0x48, 0xff),
                Color::new(0x63, 0x32, 0xc5),
            ],
            ring_count,
        )
        .colors();

        let center_column = (width.saturating_sub(1)) as f64 / 2.0;
        let center_row = (height.saturating_sub(1)) as f64 / 2.0;
        let maximum_column_radius = center_column;
        let maximum_row_radius = center_row;

        let ring_coord = |position: RingPosition, rotation: f64| {
            let direction = if position.ring % 2 == 0 { 1.0 } else { -1.0 };
            let angle = position.angle + rotation * direction;
            let column = center_column
                + angle.cos() * maximum_column_radius * position.radius_ratio;
            let row =
                center_row + angle.sin() * maximum_row_radius * position.radius_ratio;

            Coord::new(column.round() as i32, row.round() as i32)
        };

        let mut frames = Vec::new();

        // Disperse the text into concentric rings.
        const DISPERSE_FRAMES: usize = 24;
        for step in 0..=DISPERSE_FRAMES {
            let progress = easing::in_out_sine(step as f64 / DISPERSE_FRAMES as f64);
            let mut canvas = Canvas::new(width, height);

            for (index, glyph) in glyphs.iter().enumerate() {
                let ring_position = ring_positions[index];
                let target = ring_coord(ring_position, 0.0);
                let coord = glyph.input_coord.lerp(target, progress);
                let ring_color = ring_colors[ring_position.ring];
                let color = glyph.final_color.lerp(ring_color, progress);

                canvas.set(
                    coord,
                    glyph.symbol.clone(),
                    Style {
                        foreground: Some(color),
                        bold: progress > 0.65,
                        ..Style::default()
                    },
                );
            }

            frames.push(canvas.render());
        }

        // Rotate neighboring rings in opposite directions.
        const ROTATION_FRAMES: usize = 64;
        for step in 1..=ROTATION_FRAMES {
            let progress = step as f64 / ROTATION_FRAMES as f64;
            let rotation = TAU * progress;
            let mut canvas = Canvas::new(width, height);

            for (index, glyph) in glyphs.iter().enumerate() {
                let ring_position = ring_positions[index];
                let coord = ring_coord(ring_position, rotation);
                let base_color = ring_colors[ring_position.ring];
                let pulse = ((progress * TAU * 2.0 + ring_position.angle).sin() + 1.0)
                    / 2.0;
                let color = base_color.lerp(Color::new(0xff, 0xff, 0xff), pulse * 0.28);

                canvas.set(
                    coord,
                    glyph.symbol.clone(),
                    Style {
                        foreground: Some(color),
                        bold: true,
                        ..Style::default()
                    },
                );
            }

            frames.push(canvas.render());
        }

        // Return each character to its original coordinate and final gradient.
        const CONVERGE_FRAMES: usize = 24;
        for step in 1..=CONVERGE_FRAMES {
            let progress = easing::in_out_sine(step as f64 / CONVERGE_FRAMES as f64);
            let mut canvas = Canvas::new(width, height);

            for (index, glyph) in glyphs.iter().enumerate() {
                let ring_position = ring_positions[index];
                let start = ring_coord(ring_position, TAU);
                let coord = start.lerp(glyph.input_coord, progress);
                let ring_color = ring_colors[ring_position.ring];
                let color = ring_color.lerp(glyph.final_color, progress);

                canvas.set(
                    coord,
                    glyph.symbol.clone(),
                    Style {
                        foreground: Some(color),
                        bold: progress < 0.75,
                        ..Style::default()
                    },
                );
            }

            frames.push(canvas.render());
        }

        // Briefly hold the completed, colorized text.
        for _ in 0..4 {
            let mut canvas = Canvas::new(width, height);

            for glyph in &glyphs {
                canvas.set(
                    glyph.input_coord,
                    glyph.symbol.clone(),
                    Style {
                        foreground: Some(glyph.final_color),
                        ..Style::default()
                    },
                );
            }

            frames.push(canvas.render());
        }

        frames
    }
}
