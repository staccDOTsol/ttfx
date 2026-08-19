use super::Effect;
use crate::engine::{Canvas, CharacterId, EffectCharacter};
use crate::utils::easing;
use crate::utils::graphics::{Color, ColorPair, Gradient, Style};
use crate::utils::Coord;

pub struct Slide {
    movement_speed: f64,
    gap: usize,
    reverse_direction: bool,
    merge: bool,
    final_gradient_stops: Vec<Color>,
    final_gradient_steps: usize,
}

impl Slide {
    pub fn new() -> Self {
        Self {
            movement_speed: 0.5,
            gap: 3,
            reverse_direction: false,
            merge: false,
            final_gradient_stops: vec![
                Color::new(0x12, 0xa0, 0xc0),
                Color::new(0x80, 0x00, 0x80),
            ],
            final_gradient_steps: 12,
        }
    }
}

impl Default for Slide {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct SlidingCharacter {
    character: EffectCharacter,
    start: Coord,
    target: Coord,
    delay: usize,
}

impl Effect for Slide {
    fn name(&self) -> &str {
        "slide"
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
        let palette = Gradient::new(
            self.final_gradient_stops.iter().copied(),
            self.final_gradient_steps,
        )
        .colors();

        let mut characters = Vec::new();
        let mut next_id = 0_u32;

        for (row, line) in lines.iter().enumerate() {
            let from_right = if self.merge {
                row % 2 == 0
            } else {
                self.reverse_direction
            };

            for (column, symbol) in line.iter().copied().enumerate() {
                let target = Coord::new(column as i32, row as i32);
                let start = if from_right {
                    Coord::new(column as i32 + width as i32, row as i32)
                } else {
                    Coord::new(column as i32 - width as i32, row as i32)
                };

                let color_index = if palette.len() <= 1 || height <= 1 {
                    0
                } else {
                    row * (palette.len() - 1) / (height - 1)
                };
                let color = palette
                    .get(color_index)
                    .copied()
                    .unwrap_or(Color::new(0x12, 0xa0, 0xc0));

                let display_symbol = if symbol == '\t' {
                    " ".to_owned()
                } else {
                    symbol.to_string()
                };

                let mut character =
                    EffectCharacter::new(CharacterId(next_id), display_symbol, start);
                character.style =
                    Style::with_colors(ColorPair::new(Some(color), None));
                next_id += 1;

                characters.push(SlidingCharacter {
                    character,
                    start,
                    target,
                    delay: row * self.gap,
                });
            }
        }

        if characters.is_empty() {
            let mut canvas = Canvas::new(width, height);
            let color = palette
                .first()
                .copied()
                .unwrap_or(Color::new(0x12, 0xa0, 0xc0));
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style::with_colors(ColorPair::new(Some(color), None)),
            );
            return vec![canvas.render()];
        }

        let movement_frames =
            ((width as f64 / self.movement_speed.max(0.01)).ceil() as usize)
                .max(1);
        let final_frame = characters
            .iter()
            .map(|character| character.delay + movement_frames)
            .max()
            .unwrap_or(movement_frames);

        let mut frames = Vec::new();

        for frame_index in 0..=final_frame {
            let mut canvas = Canvas::new(width, height);
            let mut drew_character = false;

            for sliding in &mut characters {
                if frame_index < sliding.delay {
                    continue;
                }

                let elapsed = frame_index - sliding.delay;
                let progress =
                    (elapsed as f64 / movement_frames as f64).clamp(0.0, 1.0);
                let eased_progress = easing::in_out_quart(progress);
                sliding.character.position =
                    sliding.start.lerp(sliding.target, eased_progress);

                if canvas.draw_character(&sliding.character) {
                    drew_character = true;
                }
            }

            // Initial positions may be entirely outside the canvas. Omitting those
            // blank frames also guarantees that every returned frame contains SGR.
            if drew_character {
                frames.push(canvas.render());
            }
        }

        if frames.is_empty() {
            let mut canvas = Canvas::new(width, height);
            for sliding in &characters {
                let mut character = sliding.character.clone();
                character.position = sliding.target;
                canvas.draw_character(&character);
            }
            frames.push(canvas.render());
        }

        frames
    }
}
