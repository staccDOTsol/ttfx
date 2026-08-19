use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

pub struct Wipe;

impl Wipe {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Wipe {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Wipe {
    fn name(&self) -> &str {
        "wipe"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        const COLOR_TRANSITION_FRAMES: i32 = 4;

        let lines: Vec<Vec<char>> = if input.is_empty() {
            vec![Vec::new()]
        } else {
            input.lines().map(|line| line.chars().collect()).collect()
        };

        let width = lines
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let final_colors = Gradient::new(
            [
                Color::new(131, 58, 180),
                Color::new(253, 29, 29),
                Color::new(252, 176, 69),
            ],
            height,
        )
        .colors();

        let edge_color = Color::new(255, 255, 255);
        let hidden_style = Style {
            foreground: Some(Color::new(1, 1, 1)),
            ..Style::default()
        };

        let maximum_diagonal = width as i32 + height as i32 - 2;
        let mut frames = Vec::new();

        for front in 0..=maximum_diagonal + COLOR_TRANSITION_FRAMES {
            let mut canvas = Canvas::new(width, height);
            canvas.fill(" ", hidden_style);

            for (row, line) in lines.iter().enumerate() {
                let target_color = final_colors
                    .get(row)
                    .copied()
                    .unwrap_or(Color::new(252, 176, 69));

                for (column, symbol) in line.iter().enumerate() {
                    let diagonal = column as i32 + row as i32;

                    if diagonal > front {
                        continue;
                    }

                    let age = front - diagonal;
                    let transition = (age as f64
                        / COLOR_TRANSITION_FRAMES as f64)
                        .clamp(0.0, 1.0);
                    let color = edge_color.lerp(target_color, transition);

                    canvas.set(
                        Coord::new(column as i32, row as i32),
                        symbol.to_string(),
                        Style {
                            foreground: Some(color),
                            bold: age == 0,
                            ..Style::default()
                        },
                    );
                }
            }

            frames.push(canvas.render());
        }

        frames
    }
}
