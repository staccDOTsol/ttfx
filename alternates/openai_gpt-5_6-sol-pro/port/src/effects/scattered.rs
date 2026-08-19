
use super::Effect;
use crate::engine::{Canvas, CharacterId, EffectCharacter};
use crate::utils::easing;
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

const MOVEMENT_SPEED: f64 = 0.3;
const GRADIENT_STEPS: usize = 12;
const START_COLOR: Color = Color::new(0xff, 0x90, 0x48);
const END_COLOR: Color = Color::new(0xab, 0x48, 0xff);

pub struct Scattered;

impl Scattered {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Scattered {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Scattered {
    fn name(&self) -> &str {
        "scattered"
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

        let final_gradient =
            Gradient::new([START_COLOR, END_COLOR], GRADIENT_STEPS).colors();
        let mut rng = ScatteredRng::new(seed_from_input(input));
        let mut characters = Vec::new();
        let mut maximum_steps = 1usize;
        let mut next_id = 0u32;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let target = Coord::new(column as i32, row as i32);
                let start = Coord::new(
                    rng.bounded(width) as i32,
                    rng.bounded(height) as i32,
                );

                // The source engine treats terminal rows as twice as tall when
                // calculating path length.
                let column_delta = (target.column - start.column) as f64;
                let row_delta = 2.0 * (target.row - start.row) as f64;
                let distance = column_delta.hypot(row_delta);
                let steps = (distance / MOVEMENT_SPEED).round() as usize;
                let steps = steps.max(1);
                maximum_steps = maximum_steps.max(steps);

                let color_index = diagonal_gradient_index(
                    target,
                    width,
                    height,
                    final_gradient.len(),
                );
                let color = final_gradient[color_index];

                let mut character =
                    EffectCharacter::new(CharacterId(next_id), symbol.to_string(), start);
                next_id = next_id.wrapping_add(1);
                character.style =
                    Style::with_colors(ColorPair::new(Some(color), None));

                characters.push(ScatteredCharacter {
                    character,
                    start,
                    target,
                    steps,
                });
            }
        }

        let mut canvas = Canvas::new(width, height);
        let backdrop_style =
            Style::with_colors(ColorPair::new(Some(START_COLOR), None));
        let mut frames = Vec::with_capacity(maximum_steps);

        for current_step in 1..=maximum_steps {
            // Styling blank cells ensures even empty or temporarily off-canvas
            // scattered frames retain explicit SGR output.
            canvas.fill(" ", backdrop_style);

            for state in &mut characters {
                let progress =
                    (current_step.min(state.steps) as f64 / state.steps as f64)
                        .clamp(0.0, 1.0);
                let eased = in_out_back(progress);

                state.character.position =
                    interpolate_unclamped(state.start, state.target, eased);
                canvas.draw_character(&state.character);
            }

            frames.push(canvas.render());
        }

        if characters.is_empty() {
            canvas.fill(" ", backdrop_style);
            return vec![canvas.render()];
        }

        frames
    }
}

struct ScatteredCharacter {
    character: EffectCharacter,
    start: Coord,
    target: Coord,
    steps: usize,
}

fn input_lines(input: &str) -> Vec<String> {
    if input.is_empty() {
        return vec![String::new()];
    }

    let mut lines = input
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_owned())
        .collect::<Vec<_>>();

    if input.ends_with('\n') {
        lines.pop();
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn diagonal_gradient_index(
    coord: Coord,
    width: usize,
    height: usize,
    color_count: usize,
) -> usize {
    if color_count <= 1 {
        return 0;
    }

    let horizontal = coord.column.max(0) as usize;
    let vertical = height
        .saturating_sub(1)
        .saturating_sub(coord.row.max(0) as usize);
    let maximum = width.saturating_sub(1) + height.saturating_sub(1);

    if maximum == 0 {
        return 0;
    }

    let progress = (horizontal + vertical) as f64 / maximum as f64;
    ((color_count - 1) as f64 * progress)
        .round()
        .clamp(0.0, (color_count - 1) as f64) as usize
}

fn interpolate_unclamped(start: Coord, end: Coord, progress: f64) -> Coord {
    let column =
        start.column as f64 + (end.column - start.column) as f64 * progress;
    let row = start.row as f64 + (end.row - start.row) as f64 * progress;

    Coord::new(column.round() as i32, row.round() as i32)
}

fn in_out_back(progress: f64) -> f64 {
    let progress = easing::linear(progress);
    const C1: f64 = 1.70158;
    const C2: f64 = C1 * 1.525;

    if progress < 0.5 {
        let value = 2.0 * progress;
        value.powi(2) * ((C2 + 1.0) * value - C2) / 2.0
    } else {
        let value = 2.0 * progress - 2.0;
        (value.powi(2) * ((C2 + 1.0) * value + C2) + 2.0) / 2.0
    }
}

fn seed_from_input(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;

    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    if hash == 0 {
        0x9e37_79b9_7f4a_7c15
    } else {
        hash
    }
}

struct ScatteredRng {
    state: u64,
}

impl ScatteredRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn bounded(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            0
        } else {
            (self.next() % upper as u64) as usize
        }
    }
}
