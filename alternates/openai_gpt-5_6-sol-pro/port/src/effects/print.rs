
use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, Gradient, Style};

pub struct Print;

impl Print {
    pub fn new() -> Self {
        Self
    }

    fn render_frame(
        lines: &[Vec<char>],
        width: usize,
        height: usize,
        completed_rows: usize,
        active_row: Option<(usize, usize)>,
        print_head: Option<Coord>,
        palette: &[Color],
    ) -> String {
        let mut canvas = Canvas::new(width, height);
        let maximum_diagonal = width
            .saturating_sub(1)
            .saturating_add(height.saturating_sub(1))
            .max(1);

        let color_at = |column: usize, row: usize| {
            let diagonal = column.saturating_add(row);
            let index = diagonal
                .saturating_mul(palette.len().saturating_sub(1))
                / maximum_diagonal;
            palette[index.min(palette.len().saturating_sub(1))]
        };

        let mut styled_character_drawn = false;

        for (row, line) in lines.iter().enumerate() {
            let visible_count = if row < completed_rows {
                line.len()
            } else if let Some((active_row_index, count)) = active_row {
                if row == active_row_index {
                    count.min(line.len())
                } else {
                    0
                }
            } else {
                0
            };

            for (column, symbol) in line.iter().take(visible_count).enumerate() {
                let style = Style {
                    foreground: Some(color_at(column, row)),
                    ..Style::default()
                };

                canvas.set(
                    Coord::new(column as i32, row as i32),
                    symbol.to_string(),
                    style,
                );
                styled_character_drawn = true;
            }
        }

        if let Some(coord) = print_head {
            let head_style = Style {
                foreground: Some(Color::new(255, 255, 255)),
                bold: true,
                ..Style::default()
            };
            canvas.set(coord, "█", head_style);
            styled_character_drawn = true;
        }

        // Keep empty and blank-only inputs ANSI-styled as well.
        if !styled_character_drawn {
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style {
                    foreground: Some(palette[0]),
                    ..Style::default()
                },
            );
        }

        canvas.render()
    }
}

impl Default for Print {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Print {
    fn name(&self) -> &str {
        "print"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        let normalized = normalized.trim_end_matches('\n');

        let lines: Vec<Vec<char>> = if normalized.is_empty() {
            vec![Vec::new()]
        } else {
            normalized
                .split('\n')
                .map(|line| line.chars().collect())
                .collect()
        };

        let width = lines
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let palette = Gradient::new(
            [
                Color::new(0x02, 0xb8, 0xbd),
                Color::new(0xc1, 0xf0, 0xe3),
                Color::new(0x00, 0xff, 0xa0),
            ],
            12,
        )
        .colors();

        let mut frames = Vec::new();
        let mut completed_rows = 0;

        for (row, line) in lines.iter().enumerate() {
            if line.is_empty() {
                frames.push(Self::render_frame(
                    &lines,
                    width,
                    height,
                    completed_rows,
                    Some((row, 0)),
                    Some(Coord::new(0, row as i32)),
                    &palette,
                ));
            } else {
                for column in 0..line.len() {
                    frames.push(Self::render_frame(
                        &lines,
                        width,
                        height,
                        completed_rows,
                        Some((row, column)),
                        Some(Coord::new(column as i32, row as i32)),
                        &palette,
                    ));
                }
            }

            completed_rows = row + 1;

            // Simulate the printer's carriage return. The head accelerates
            // toward the left edge and decelerates before beginning the next row.
            let return_distance = line.len().saturating_sub(1);
            let return_frames = return_distance.div_ceil(3);

            for step in 1..=return_frames {
                let progress = step as f64 / return_frames as f64;
                let eased = easing::in_out_quad(progress);
                let column =
                    (return_distance as f64 * (1.0 - eased)).round() as i32;

                frames.push(Self::render_frame(
                    &lines,
                    width,
                    height,
                    completed_rows,
                    None,
                    Some(Coord::new(column, row as i32)),
                    &palette,
                ));
            }
        }

        frames.push(Self::render_frame(
            &lines,
            width,
            height,
            lines.len(),
            None,
            None,
            &palette,
        ));

        frames
    }
}
