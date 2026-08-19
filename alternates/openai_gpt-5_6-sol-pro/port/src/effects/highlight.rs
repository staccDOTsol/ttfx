
use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

const HIGHLIGHT_COLOR: Color = Color::new(255, 255, 255);
const HIGHLIGHT_DURATION: usize = 2;
const HIGHLIGHT_STEPS: usize = 5;
const FINAL_GRADIENT_STEPS: usize = 12;

pub struct Highlight;

impl Highlight {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Highlight {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Highlight {
    fn name(&self) -> &str {
        "highlight"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let input = input.trim_end_matches(&['\r', '\n'][..]);
        let lines: Vec<Vec<char>> = if input.is_empty() {
            vec![Vec::new()]
        } else {
            input
                .split('\n')
                .map(|line| line.trim_end_matches('\r').chars().collect())
                .collect()
        };

        let width = lines.iter().map(Vec::len).max().unwrap_or(0).max(1);
        let height = lines.len().max(1);
        let has_characters = lines.iter().any(|line| !line.is_empty());

        let final_palette = Gradient::new(
            [
                Color::new(0x8a, 0x00, 0x8a),
                Color::new(0x00, 0xd1, 0xff),
                Color::new(0xff, 0xff, 0xff),
            ],
            FINAL_GRADIENT_STEPS,
        )
        .colors();

        let base_colors: Vec<Color> = (0..height)
            .map(|row| {
                let palette_index = if height == 1 {
                    0
                } else {
                    row * (final_palette.len() - 1) / (height - 1)
                };
                final_palette[palette_index]
            })
            .collect();

        if !has_characters {
            let mut canvas = Canvas::new(1, 1);
            canvas.fill(
                " ",
                Style {
                    foreground: Some(base_colors[0]),
                    ..Style::default()
                },
            );
            return vec![canvas.render()];
        }

        let transition_palettes: Vec<Vec<Color>> = base_colors
            .iter()
            .map(|base| {
                Gradient::new(
                    [*base, HIGHLIGHT_COLOR, *base],
                    HIGHLIGHT_STEPS,
                )
                .colors()
            })
            .collect();

        let final_diagonal = lines
            .iter()
            .enumerate()
            .flat_map(|(row, line)| {
                line.iter()
                    .enumerate()
                    .map(move |(column, _)| row + column)
            })
            .max()
            .unwrap_or(0);

        let animation_length = HIGHLIGHT_STEPS * HIGHLIGHT_DURATION;
        let frame_count = final_diagonal + animation_length;
        let mut frames = Vec::with_capacity(frame_count);

        for tick in 0..frame_count {
            let mut canvas = Canvas::new(width, height);

            for (row, line) in lines.iter().enumerate() {
                for (column, symbol) in line.iter().enumerate() {
                    let activation_tick = row + column;
                    let color = if tick >= activation_tick {
                        let local_tick = tick - activation_tick;
                        let color_index = local_tick / HIGHLIGHT_DURATION;

                        transition_palettes[row]
                            .get(color_index)
                            .copied()
                            .unwrap_or(base_colors[row])
                    } else {
                        base_colors[row]
                    };

                    canvas.set(
                        Coord::new(column as i32, row as i32),
                        symbol.to_string(),
                        Style {
                            foreground: Some(color),
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
