
use std::collections::VecDeque;

use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

pub struct Overflow;

impl Overflow {
    pub fn new() -> Self {
        Self
    }

    fn render_rows(
        rows: &VecDeque<Vec<String>>,
        width: usize,
        height: usize,
        colors: &[Color],
    ) -> String {
        let mut canvas = Canvas::new(width, height);
        let empty_row = vec![" ".to_owned(); width];

        for screen_row in 0..height {
            let row = rows.get(screen_row).unwrap_or(&empty_row);
            let color = colors
                .get(screen_row)
                .copied()
                .or_else(|| colors.last().copied())
                .unwrap_or(Color::new(255, 255, 255));

            let style = Style {
                foreground: Some(color),
                ..Style::default()
            };

            for column in 0..width {
                let symbol = row.get(column).map(String::as_str).unwrap_or(" ");
                canvas.set(
                    Coord::new(column as i32, screen_row as i32),
                    symbol,
                    style,
                );
            }
        }

        canvas.render()
    }

    fn parse_rows(input: &str) -> (Vec<Vec<String>>, usize) {
        let mut lines: Vec<&str> = input.lines().collect();

        if lines.is_empty() {
            lines.push("");
        }

        let width = lines
            .iter()
            .map(|line| line.trim_end_matches('\r').chars().count())
            .max()
            .unwrap_or(0)
            .max(1);

        let rows = lines
            .into_iter()
            .map(|line| {
                let mut row = line
                    .trim_end_matches('\r')
                    .chars()
                    .map(|character| character.to_string())
                    .collect::<Vec<_>>();
                row.resize(width, " ".to_owned());
                row
            })
            .collect();

        (rows, width)
    }
}

impl Default for Overflow {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Overflow {
    fn name(&self) -> &str {
        "overflow"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let (source_rows, width) = Self::parse_rows(input);
        let height = source_rows.len().max(1);

        let overflow_colors = Gradient::new(
            [
                Color::new(242, 235, 192),
                Color::new(141, 191, 179),
                Color::new(82, 148, 168),
                Color::new(242, 235, 192),
            ],
            height,
        )
        .colors();

        let final_colors = Gradient::new(
            [
                Color::new(138, 0, 138),
                Color::new(0, 209, 255),
                Color::new(255, 255, 255),
            ],
            height,
        )
        .colors();

        let blank_row = vec![" ".to_owned(); width];
        let mut visible_rows =
            VecDeque::from(vec![blank_row; height]);
        let mut frames = Vec::new();

        // Feed several copies upward through the canvas. The final copy is kept
        // in its original order so the animation settles on the source text.
        for cycle in 0..3 {
            for offset in 0..height {
                let source_index = if cycle < 2 {
                    (offset + cycle) % height
                } else {
                    offset
                };

                visible_rows.pop_front();
                visible_rows.push_back(source_rows[source_index].clone());

                let shifted_colors = (0..height)
                    .map(|row| {
                        let index = (row + cycle + offset) % height;
                        overflow_colors[index]
                    })
                    .collect::<Vec<_>>();

                frames.push(Self::render_rows(
                    &visible_rows,
                    width,
                    height,
                    &shifted_colors,
                ));
            }
        }

        // Fade the moving overflow palette into the final vertical gradient.
        let transition_steps = 8;
        for step in 1..=transition_steps {
            let progress = step as f64 / transition_steps as f64;
            let colors = (0..height)
                .map(|row| {
                    overflow_colors[row]
                        .lerp(final_colors[row], progress)
                })
                .collect::<Vec<_>>();

            frames.push(Self::render_rows(
                &visible_rows,
                width,
                height,
                &colors,
            ));
        }

        frames
    }
}
