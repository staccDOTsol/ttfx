use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

pub struct Pour;

impl Pour {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Pour {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Pour {
    fn name(&self) -> &str {
        "pour"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut lines: Vec<&str> = input.split('\n').collect();
        if input.ends_with('\n') {
            lines.pop();
        }
        if lines.is_empty() {
            lines.push("");
        }

        let lines: Vec<&str> = lines
            .into_iter()
            .map(|line| line.trim_end_matches('\r'))
            .collect();

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
            height,
        )
        .colors();
        let starting_color = Color::new(0xf4, 0xfb, 0xff);

        struct Drop {
            symbol: String,
            column: usize,
            target_row: usize,
            delay: usize,
            duration: usize,
            final_color: Color,
        }

        let mut drops = Vec::new();

        // Characters destined for lower rows enter first, so each column
        // appears to fill from the bottom upward.
        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let distance = row;
                let delay = (height - 1 - row) * 2 + column % 3;
                let duration = (distance * 3).max(1);

                drops.push(Drop {
                    symbol: symbol.to_string(),
                    column,
                    target_row: row,
                    delay,
                    duration,
                    final_color: gradient
                        .get(row)
                        .copied()
                        .unwrap_or(Color::new(0x00, 0xd1, 0xff)),
                });
            }
        }

        if drops.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style::with_colors(ColorPair::new(
                    Some(starting_color),
                    None,
                )),
            );
            return vec![canvas.render()];
        }

        // Ensure the first emitted frame already contains styled content.
        let first_delay = drops.iter().map(|drop| drop.delay).min().unwrap_or(0);
        for drop in &mut drops {
            drop.delay -= first_delay;
        }

        let last_frame = drops
            .iter()
            .map(|drop| drop.delay + drop.duration)
            .max()
            .unwrap_or(0);

        let mut frames = Vec::with_capacity(last_frame + 1);

        for tick in 0..=last_frame {
            let mut canvas = Canvas::new(width, height);

            for drop in &drops {
                if tick < drop.delay {
                    continue;
                }

                let elapsed = (tick - drop.delay).min(drop.duration);
                let progress = elapsed as f64 / drop.duration as f64;
                let eased = easing::out_bounce(progress);
                let row = (drop.target_row as f64 * eased).round() as i32;
                let color = starting_color.lerp(drop.final_color, progress);

                canvas.set(
                    Coord::new(drop.column as i32, row),
                    drop.symbol.clone(),
                    Style::with_colors(ColorPair::new(Some(color), None)),
                );
            }

            frames.push(canvas.render());
        }

        frames
    }
}
