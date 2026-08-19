
use super::Effect;
use crate::engine::{Canvas, CharacterId, EffectCharacter};
use crate::utils::easing;
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

const MOVEMENT_SPEED: f64 = 0.35;
const GRADIENT_STEPS: usize = 12;

#[derive(Clone, Debug)]
struct ExpandingCharacter {
    character: EffectCharacter,
    target: Coord,
    total_steps: usize,
}

pub struct Expand;

impl Expand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Expand {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Expand {
    fn name(&self) -> &str {
        "expand"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let normalized = input.trim_end_matches(['\r', '\n']);
        if normalized.is_empty() {
            return Vec::new();
        }

        let lines = normalized
            .split('\n')
            .map(|line| line.trim_end_matches('\r').chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let width = lines.iter().map(Vec::len).max().unwrap_or(0);
        let height = lines.len();

        if width == 0 || height == 0 {
            return Vec::new();
        }

        let center = Coord::new(
            ((width - 1) / 2) as i32,
            (height / 2) as i32,
        );

        let palette = Gradient::new(
            [
                Color::new(0x8a, 0x00, 0x8a),
                Color::new(0x00, 0xd1, 0xff),
                Color::new(0xff, 0xff, 0xff),
            ],
            GRADIENT_STEPS,
        )
        .colors();

        let mut characters = Vec::new();
        let mut next_id = 0_u32;
        let mut maximum_steps = 0_usize;

        for (row, line) in lines.iter().enumerate() {
            let color_progress = if height <= 1 {
                0.0
            } else {
                (height - 1 - row) as f64 / (height - 1) as f64
            };
            let color_index = round_half_even(
                color_progress * (palette.len().saturating_sub(1)) as f64,
            )
            .clamp(0, palette.len().saturating_sub(1) as i64)
                as usize;

            let style = Style::with_colors(ColorPair::new(
                Some(palette[color_index]),
                None,
            ));

            for (column, symbol) in line.iter().enumerate() {
                let target = Coord::new(column as i32, row as i32);
                let dx = (target.column - center.column) as f64;
                let dy = (target.row - center.row) as f64 * 2.0;
                let distance = dx.hypot(dy);

                let total_steps = if distance <= f64::EPSILON {
                    1
                } else {
                    round_half_even(distance / MOVEMENT_SPEED).max(1) as usize
                };

                maximum_steps = maximum_steps.max(total_steps);

                let mut character =
                    EffectCharacter::new(CharacterId(next_id), symbol.to_string(), center);
                character.style = style;
                next_id = next_id.saturating_add(1);

                characters.push(ExpandingCharacter {
                    character,
                    target,
                    total_steps,
                });
            }
        }

        if characters.is_empty() {
            return Vec::new();
        }

        let mut canvas = Canvas::new(width, height);
        let mut frames = Vec::with_capacity(maximum_steps);

        for step in 1..=maximum_steps {
            canvas.clear();

            for expanding in &mut characters {
                let progress =
                    (step.min(expanding.total_steps) as f64 / expanding.total_steps as f64)
                        .clamp(0.0, 1.0);
                let eased_progress = easing::in_out_quart(progress);

                expanding.character.position = interpolate_coord(
                    center,
                    expanding.target,
                    eased_progress,
                );
                canvas.draw_character(&expanding.character);
            }

            frames.push(canvas.render());
        }

        frames
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
    if !value.is_finite() {
        return 0;
    }

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
