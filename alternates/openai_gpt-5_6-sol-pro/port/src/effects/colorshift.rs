
use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

#[derive(Clone, Debug)]
pub struct Colorshift {
    gradient_steps: usize,
    frames_per_color: usize,
}

impl Colorshift {
    pub fn new() -> Self {
        Self {
            gradient_steps: 12,
            frames_per_color: 5,
        }
    }
}

impl Default for Colorshift {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Colorshift {
    fn name(&self) -> &str {
        "colorshift"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut lines: Vec<Vec<char>> = input
            .lines()
            .map(|line| line.trim_end_matches('\r').chars().collect())
            .collect();

        if lines.is_empty() {
            lines.push(Vec::new());
        }

        let width = lines
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        // The repeated first stop closes the gradient smoothly when the
        // animation cycles from its final frame back to its first.
        let stops = [
            Color::new(0xe8, 0x14, 0x16),
            Color::new(0xff, 0xa5, 0x00),
            Color::new(0xfa, 0xeb, 0x36),
            Color::new(0x79, 0xc3, 0x14),
            Color::new(0x48, 0x7d, 0xe7),
            Color::new(0x4b, 0x36, 0x9d),
            Color::new(0x70, 0x36, 0x9d),
            Color::new(0xe8, 0x14, 0x16),
        ];
        let color_count =
            self.gradient_steps.max(1) * (stops.len() - 1);
        let colors = Gradient::new(stops, color_count).colors();

        let mut frames =
            Vec::with_capacity(colors.len() * self.frames_per_color.max(1));

        for color in colors {
            let style = Style {
                foreground: Some(color),
                ..Style::default()
            };
            let mut canvas = Canvas::new(width, height);
            let mut drew_character = false;

            for (row, line) in lines.iter().enumerate() {
                for (column, symbol) in line.iter().enumerate() {
                    canvas.set(
                        Coord::new(column as i32, row as i32),
                        symbol.to_string(),
                        style,
                    );
                    drew_character = true;
                }
            }

            // Keep even empty input ANSI-styled so every emitted frame remains
            // a genuine color-shift frame rather than unstyled plain text.
            if !drew_character {
                canvas.set(Coord::new(0, 0), " ", style);
            }

            let rendered = canvas.render();
            for _ in 0..self.frames_per_color.max(1) {
                frames.push(rendered.clone());
            }
        }

        frames
    }
}
