
use super::Effect;
use crate::engine::{Canvas, CharacterId, EffectCharacter};
use crate::utils::easing;
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

const MOVEMENT_SPEED: f64 = 0.15;
const GRADIENT_STEPS: usize = 12;
const GRADIENT_START: Color = Color::new(0x8a, 0x00, 0x8a);
const GRADIENT_END: Color = Color::new(0x00, 0xd1, 0xff);

#[derive(Clone, Debug)]
struct SliceCharacter {
    character: EffectCharacter,
    start: Coord,
    target: Coord,
    total_steps: usize,
}

pub struct Slice;

impl Slice {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Slice {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Slice {
    fn name(&self) -> &str {
        "slice"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines = input_lines(input);
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);
        let center_column = (width.saturating_sub(1) / 2) as i32;

        let palette =
            Gradient::new([GRADIENT_START, GRADIENT_END], GRADIENT_STEPS)
                .colors();

        let mut moving_characters = Vec::new();
        let mut next_id = 0_u32;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let target = Coord::new(column as i32, row as i32);

                // The left half is sliced in from above while the right half
                // is sliced in from below, matching the Python effect's
                // default vertical slicing mode.
                let start = if target.column <= center_column {
                    Coord::new(target.column, -1)
                } else {
                    Coord::new(target.column, height as i32)
                };

                let distance = (target.row - start.row).abs() as f64 * 2.0;
                let total_steps =
                    round_half_even(distance / MOVEMENT_SPEED).max(1) as usize;

                let color_progress = if height <= 1 {
                    0.0
                } else {
                    (height - 1 - row) as f64 / (height - 1) as f64
                };
                let color_index = round_half_even(
                    color_progress * palette.len().saturating_sub(1) as f64,
                )
                .clamp(0, palette.len().saturating_sub(1) as i64)
                    as usize;

                let mut character = EffectCharacter::new(
                    CharacterId(next_id),
                    symbol.to_string(),
                    start,
                );
                next_id = next_id.saturating_add(1);
                character.style = Style::with_colors(ColorPair::new(
                    palette.get(color_index).copied(),
                    None,
                ));

                moving_characters.push(SliceCharacter {
                    character,
                    start,
                    target,
                    total_steps,
                });
            }
        }

        let maximum_steps = moving_characters
            .iter()
            .map(|character| character.total_steps)
            .max()
            .unwrap_or(1);

        let anchor_style = Style::with_colors(ColorPair::new(
            palette.first().copied().or(Some(GRADIENT_START)),
            None,
        ));
        let mut canvas = Canvas::new(width, height);
        let mut frames = Vec::with_capacity(maximum_steps);

        for frame_index in 1..=maximum_steps {
            canvas.clear();

            // Keep even an otherwise blank frame explicitly styled so every
            // rendered frame carries an SGR sequence.
            canvas.set(Coord::new(0, 0), " ", anchor_style);

            for moving in &mut moving_characters {
                let step = frame_index.min(moving.total_steps);
                let progress = step as f64 / moving.total_steps as f64;
                let eased_progress = easing::in_out_expo(progress);

                moving.character.position = interpolate_coord(
                    moving.start,
                    moving.target,
                    eased_progress,
                );
                canvas.draw_character(&moving.character);
            }

            frames.push(canvas.render());
        }

        frames
    }
}

fn input_lines(input: &str) -> Vec<&str> {
    let input = input.trim_end_matches(['\n', '\r']);

    if input.is_empty() {
        vec![""]
    } else {
        input.lines().collect()
    }
}

fn interpolate_coord(start: Coord, end: Coord, progress: f64) -> Coord {
    let progress = progress.clamp(0.0, 1.0);
    let column =
        start.column as f64 + (end.column - start.column) as f64 * progress;
    let row = start.row as f64 + (end.row - start.row) as f64 * progress;

    Coord::new(
        round_half_even(column) as i32,
        round_half_even(row) as i32,
    )
}

fn round_half_even(value: f64) -> i64 {
    let lower = value.floor();
    let fraction = value - lower;

    if fraction < 0.5 {
        lower as i64
    } else if fraction > 0.5 {
        lower as i64 + 1
    } else {
        let lower_integer = lower as i64;
        if lower_integer % 2 == 0 {
            lower_integer
        } else {
            lower_integer + 1
        }
    }
}
