
use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

const BEAM_GRADIENT_FRAMES: usize = 2;
const FINAL_GRADIENT_FRAMES: usize = 5;
const BEAM_DELAY: usize = 2;
const TRAIL_LENGTH: i32 = 4;

pub struct Beams;

impl Beams {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Beams {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Beams {
    fn name(&self) -> &str {
        "beams"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let parsed = ParsedText::new(input);
        let width = parsed.width;
        let height = parsed.height;

        let beam_colors = Gradient::new(
            [
                Color::new(255, 255, 255),
                Color::new(0, 209, 255),
                Color::new(138, 0, 138),
            ],
            8,
        )
        .colors();

        let final_colors = Gradient::new(
            [
                Color::new(138, 0, 138),
                Color::new(0, 209, 255),
                Color::new(255, 255, 255),
            ],
            12,
        )
        .colors();

        let mut rng = Rng::new(seed_from_input(input));
        let mut beams = Vec::with_capacity(width + height);

        for row in 0..height {
            let direction = if rng.next_bool() { 1 } else { -1 };
            beams.push(Beam {
                orientation: Orientation::Horizontal,
                axis: row as i32,
                position: if direction > 0 {
                    0
                } else {
                    width.saturating_sub(1) as i32
                },
                direction,
                speed: 1 + rng.range(4) as i32,
                start_frame: 0,
                color: beam_colors[rng.range(beam_colors.len())],
                complete: false,
            });
        }

        for column in 0..width {
            let direction = if rng.next_bool() { 1 } else { -1 };
            beams.push(Beam {
                orientation: Orientation::Vertical,
                axis: column as i32,
                position: if direction > 0 {
                    0
                } else {
                    height.saturating_sub(1) as i32
                },
                direction,
                speed: 1 + rng.range(2) as i32,
                start_frame: 0,
                color: beam_colors[rng.range(beam_colors.len())],
                complete: false,
            });
        }

        shuffle(&mut beams, &mut rng);
        for (index, beam) in beams.iter_mut().enumerate() {
            beam.start_frame = index * BEAM_DELAY;
        }

        let mut illumination = vec![None; width * height];
        let mut final_started = vec![None; width * height];
        let mut frames = Vec::new();
        let mut phase = Phase::Beams;
        let mut phase_frame = 0usize;

        let maximum_frames = beams.len() * BEAM_DELAY
            + width.max(height) * 3
            + width
            + height
            + 128;

        for frame_number in 0..maximum_frames {
            let mut visible_beams = Vec::new();

            match phase {
                Phase::Beams => {
                    for beam in &mut beams {
                        if beam.complete || frame_number < beam.start_frame {
                            continue;
                        }

                        let old_position = beam.position;
                        let new_position =
                            old_position + beam.direction * beam.speed;

                        illuminate_between(
                            &parsed,
                            &mut illumination,
                            beam.orientation,
                            beam.axis,
                            old_position,
                            new_position,
                        );

                        visible_beams.push(VisibleBeam {
                            orientation: beam.orientation,
                            axis: beam.axis,
                            position: old_position,
                            direction: beam.direction,
                            color: beam.color,
                        });

                        beam.position = new_position;

                        let extent = match beam.orientation {
                            Orientation::Horizontal => width as i32,
                            Orientation::Vertical => height as i32,
                        };

                        beam.complete = if beam.direction > 0 {
                            beam.position - TRAIL_LENGTH >= extent
                        } else {
                            beam.position + TRAIL_LENGTH < 0
                        };
                    }

                    if beams.iter().all(|beam| beam.complete) {
                        phase = Phase::FinalWipe;
                        phase_frame = 0;
                    }
                }
                Phase::FinalWipe => {
                    let threshold = phase_frame.saturating_mul(2);

                    for row in 0..height {
                        for column in 0..width {
                            let index = row * width + column;
                            if parsed.present[index]
                                && column + row <= threshold
                                && final_started[index].is_none()
                            {
                                final_started[index] = Some(phase_frame);
                            }
                        }
                    }

                    phase_frame += 1;
                }
            }

            let mut canvas = Canvas::new(width, height);

            for row in 0..height {
                for column in 0..width {
                    let index = row * width + column;
                    if !parsed.present[index] {
                        continue;
                    }

                    let coord = Coord::new(column as i32, row as i32);
                    let symbol = parsed.symbols[index].to_string();

                    if let Some(started) = final_started[index] {
                        let elapsed = phase_frame.saturating_sub(started);
                        let progress =
                            elapsed as f64 / FINAL_GRADIENT_FRAMES as f64;
                        let target = final_color(
                            &final_colors,
                            column,
                            row,
                            width,
                            height,
                        );
                        let color = beam_colors[beam_colors.len() - 1]
                            .lerp(target, progress);

                        canvas.set(
                            coord,
                            symbol,
                            Style {
                                foreground: Some(color),
                                bold: elapsed < FINAL_GRADIENT_FRAMES,
                                ..Style::default()
                            },
                        );
                    } else if let Some(age) = illumination[index] {
                        let color_index =
                            (age / BEAM_GRADIENT_FRAMES)
                                .min(beam_colors.len() - 1);

                        canvas.set(
                            coord,
                            symbol,
                            Style {
                                foreground: Some(beam_colors[color_index]),
                                bold: age < BEAM_GRADIENT_FRAMES * 2,
                                ..Style::default()
                            },
                        );

                        illumination[index] = Some(age.saturating_add(1));
                    }
                }
            }

            for beam in visible_beams {
                draw_beam(&mut canvas, beam, width, height);
            }

            ensure_styled_frame(&mut canvas, width, height, &beam_colors);
            frames.push(canvas.render());

            if phase == Phase::FinalWipe {
                let all_started = parsed
                    .present
                    .iter()
                    .enumerate()
                    .all(|(index, present)| {
                        !present || final_started[index].is_some()
                    });

                let all_complete = final_started
                    .iter()
                    .flatten()
                    .all(|started| {
                        phase_frame.saturating_sub(*started)
                            > FINAL_GRADIENT_FRAMES
                    });

                if all_started && all_complete {
                    break;
                }
            }
        }

        frames
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Beams,
    FinalWipe,
}

#[derive(Clone, Copy, Debug)]
struct Beam {
    orientation: Orientation,
    axis: i32,
    position: i32,
    direction: i32,
    speed: i32,
    start_frame: usize,
    color: Color,
    complete: bool,
}

#[derive(Clone, Copy, Debug)]
struct VisibleBeam {
    orientation: Orientation,
    axis: i32,
    position: i32,
    direction: i32,
    color: Color,
}

#[derive(Debug)]
struct ParsedText {
    width: usize,
    height: usize,
    symbols: Vec<char>,
    present: Vec<bool>,
}

impl ParsedText {
    fn new(input: &str) -> Self {
        let input = input.trim_end_matches(&['\r', '\n'][..]);
        let mut lines: Vec<Vec<char>> = input
            .split('\n')
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
        let mut symbols = vec![' '; width * height];
        let mut present = vec![false; width * height];

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.iter().copied().enumerate() {
                let index = row * width + column;
                symbols[index] = symbol;
                present[index] = true;
            }
        }

        if !present.iter().any(|value| *value) {
            present[0] = true;
        }

        Self {
            width,
            height,
            symbols,
            present,
        }
    }
}

fn illuminate_between(
    parsed: &ParsedText,
    illumination: &mut [Option<usize>],
    orientation: Orientation,
    axis: i32,
    start: i32,
    end: i32,
) {
    let lower = start.min(end);
    let upper = start.max(end);

    for position in lower..=upper {
        let (column, row) = match orientation {
            Orientation::Horizontal => (position, axis),
            Orientation::Vertical => (axis, position),
        };

        if column < 0
            || row < 0
            || column >= parsed.width as i32
            || row >= parsed.height as i32
        {
            continue;
        }

        let index = row as usize * parsed.width + column as usize;
        if parsed.present[index] {
            illumination[index] = Some(0);
        }
    }
}

fn draw_beam(
    canvas: &mut Canvas,
    beam: VisibleBeam,
    width: usize,
    height: usize,
) {
    const HORIZONTAL_SYMBOLS: [&str; 5] = ["▂", "▁", "_", "─", "·"];
    const VERTICAL_SYMBOLS: [&str; 5] = ["▌", "▍", "▎", "▏", "│"];

    for distance in 0..=TRAIL_LENGTH {
        let position = beam.position - beam.direction * distance;
        let coord = match beam.orientation {
            Orientation::Horizontal => Coord::new(position, beam.axis),
            Orientation::Vertical => Coord::new(beam.axis, position),
        };

        if coord.column < 0
            || coord.row < 0
            || coord.column >= width as i32
            || coord.row >= height as i32
        {
            continue;
        }

        let progress = distance as f64 / (TRAIL_LENGTH + 1) as f64;
        let color = beam
            .color
            .lerp(Color::new(24, 0, 32), progress);
        let symbols = match beam.orientation {
            Orientation::Horizontal => &HORIZONTAL_SYMBOLS,
            Orientation::Vertical => &VERTICAL_SYMBOLS,
        };

        canvas.set(
            coord,
            symbols[distance as usize],
            Style {
                foreground: Some(color),
                bold: distance == 0,
                ..Style::default()
            },
        );
    }
}

fn final_color(
    colors: &[Color],
    column: usize,
    row: usize,
    width: usize,
    height: usize,
) -> Color {
    let denominator = width
        .saturating_sub(1)
        .saturating_add(height.saturating_sub(1));

    let progress = if denominator == 0 {
        1.0
    } else {
        (column + row) as f64 / denominator as f64
    };

    let index = (progress * colors.len().saturating_sub(1) as f64)
        .round() as usize;
    colors[index.min(colors.len() - 1)]
}

fn ensure_styled_frame(
    canvas: &mut Canvas,
    width: usize,
    height: usize,
    colors: &[Color],
) {
    let coord = Coord::new(
        width.saturating_sub(1) as i32,
        height.saturating_sub(1) as i32,
    );

    if let Some(cell) = canvas.get(coord) {
        if cell.style == Style::default() {
            let symbol = cell.symbol.clone();
            canvas.set(
                coord,
                symbol,
                Style {
                    foreground: Some(colors[0]),
                    ..Style::default()
                },
            );
        }
    }
}

fn seed_from_input(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;

    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    if hash == 0 {
        0x9e3779b97f4a7c15
    } else {
        hash
    }
}

fn shuffle<T>(values: &mut [T], rng: &mut Rng) {
    for index in (1..values.len()).rev() {
        let replacement = rng.range(index + 1);
        values.swap(index, replacement);
    }
}

#[derive(Clone, Copy, Debug)]
struct Rng {
    state: u64,
}

impl Rng {
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

    fn range(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            0
        } else {
            (self.next() % upper as u64) as usize
        }
    }

    fn next_bool(&mut self) -> bool {
        self.next() & 1 == 0
    }
}
