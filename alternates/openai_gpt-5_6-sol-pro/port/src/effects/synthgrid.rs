
use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::{Color, Gradient, Style};

pub struct Synthgrid;

impl Synthgrid {
    pub fn new() -> Self {
        Self
    }

    fn input_cells(input: &str) -> Vec<Vec<char>> {
        let input = input.trim_end_matches(|character| {
            character == '\n' || character == '\r'
        });

        if input.is_empty() {
            return vec![Vec::new()];
        }

        input
            .split('\n')
            .map(|line| line.trim_end_matches('\r').chars().collect())
            .collect()
    }

    fn grid_symbol(column: usize, row: usize, height: usize) -> &'static str {
        let vertical = column % 4 == 0;
        let horizontal = (height.saturating_sub(1).saturating_sub(row)) % 2 == 0;

        match (vertical, horizontal) {
            (true, true) => "┼",
            (true, false) => "│",
            (false, true) => "─",
            (false, false) => "·",
        }
    }

    fn gradient_color(
        colors: &[Color],
        column: usize,
        row: usize,
        width: usize,
        height: usize,
    ) -> Color {
        let maximum = width.saturating_sub(1) + height.saturating_sub(1);
        let offset = column + height.saturating_sub(1).saturating_sub(row);
        let index = if maximum == 0 {
            0
        } else {
            offset * colors.len().saturating_sub(1) / maximum
        };

        colors[index.min(colors.len().saturating_sub(1))]
    }

    fn style(color: Color, bold: bool) -> Style {
        Style {
            foreground: Some(color),
            bold,
            ..Style::default()
        }
    }
}

impl Default for Synthgrid {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Synthgrid {
    fn name(&self) -> &str {
        "synthgrid"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let cells = Self::input_cells(input);
        let height = cells.len().max(1);
        let width = cells
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(1);

        let grid_gradient = Gradient::new(
            [
                Color::new(0x35, 0x05, 0x73),
                Color::new(0x8a, 0x16, 0xd1),
                Color::new(0xff, 0x2b, 0xd6),
                Color::new(0x00, 0xd9, 0xff),
            ],
            width + height - 1,
        )
        .colors();

        let text_gradient = Gradient::new(
            [
                Color::new(0x00, 0xff, 0xff),
                Color::new(0x84, 0xff, 0xf5),
                Color::new(0xff, 0x66, 0xe8),
                Color::new(0xff, 0x2b, 0x8a),
            ],
            width + height - 1,
        )
        .colors();

        let dark = Color::new(0x18, 0x04, 0x31);
        let white = Color::new(0xff, 0xff, 0xff);
        let mut frames = Vec::new();

        let grid_frames = (width + height).clamp(8, 24);
        for frame_index in 0..grid_frames {
            let denominator = grid_frames.saturating_sub(1).max(1) as f64;
            let progress = easing::out_expo(frame_index as f64 / denominator);
            let mut canvas = Canvas::new(width, height);

            for row in 0..height {
                for column in 0..width {
                    let vertical_distance = if height <= 1 {
                        0.0
                    } else {
                        (height - 1 - row) as f64 / (height - 1) as f64
                    };
                    let center = (width.saturating_sub(1)) as f64 / 2.0;
                    let horizontal_distance = if center <= f64::EPSILON {
                        0.0
                    } else {
                        (column as f64 - center).abs() / center
                    };

                    let activation =
                        vertical_distance * 0.62 + horizontal_distance * 0.38;
                    let base_color = Self::gradient_color(
                        &grid_gradient,
                        column,
                        row,
                        width,
                        height,
                    );
                    let visible = activation <= progress;
                    let color = if visible {
                        dark.lerp(base_color, 0.45 + progress * 0.55)
                    } else {
                        dark
                    };
                    let symbol = if visible {
                        Self::grid_symbol(column, row, height)
                    } else {
                        " "
                    };

                    canvas.set(
                        crate::utils::Coord::new(column as i32, row as i32),
                        symbol,
                        Self::style(color, visible),
                    );
                }
            }

            frames.push(canvas.render());
        }

        let reveal_frames = (width + height).clamp(10, 30);
        for frame_index in 0..reveal_frames {
            let denominator = reveal_frames.saturating_sub(1).max(1) as f64;
            let progress =
                easing::in_out_cubic(frame_index as f64 / denominator);
            let mut canvas = Canvas::new(width, height);

            for row in 0..height {
                for column in 0..width {
                    let vertical = if height <= 1 {
                        0.0
                    } else {
                        (height - 1 - row) as f64 / (height - 1) as f64
                    };
                    let center = (width.saturating_sub(1)) as f64 / 2.0;
                    let horizontal = if center <= f64::EPSILON {
                        0.0
                    } else {
                        (column as f64 - center).abs() / center
                    };
                    let reveal_at = vertical * 0.72 + horizontal * 0.28;

                    let grid_color = Self::gradient_color(
                        &grid_gradient,
                        column,
                        row,
                        width,
                        height,
                    );
                    let text_color = Self::gradient_color(
                        &text_gradient,
                        column,
                        row,
                        width,
                        height,
                    );

                    let input_symbol = cells
                        .get(row)
                        .and_then(|line| line.get(column))
                        .copied()
                        .unwrap_or(' ');

                    if reveal_at <= progress {
                        canvas.set(
                            crate::utils::Coord::new(
                                column as i32,
                                row as i32,
                            ),
                            input_symbol.to_string(),
                            Self::style(
                                grid_color.lerp(text_color, progress),
                                true,
                            ),
                        );
                    } else {
                        canvas.set(
                            crate::utils::Coord::new(
                                column as i32,
                                row as i32,
                            ),
                            Self::grid_symbol(column, row, height),
                            Self::style(dark.lerp(grid_color, 0.55), false),
                        );
                    }
                }
            }

            frames.push(canvas.render());
        }

        for frame_index in 0..6 {
            let progress = frame_index as f64 / 5.0;
            let glow = (std::f64::consts::PI * progress).sin() * 0.35;
            let mut canvas = Canvas::new(width, height);

            for row in 0..height {
                for column in 0..width {
                    let symbol = cells
                        .get(row)
                        .and_then(|line| line.get(column))
                        .copied()
                        .unwrap_or(' ');

                    let color = Self::gradient_color(
                        &text_gradient,
                        column,
                        row,
                        width,
                        height,
                    )
                    .lerp(white, glow);

                    canvas.set(
                        crate::utils::Coord::new(column as i32, row as i32),
                        symbol.to_string(),
                        Self::style(color, true),
                    );
                }
            }

            frames.push(canvas.render());
        }

        frames
    }
}
