use super::Effect;

use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::geometry::{quadratic_bezier, Coord};
use crate::utils::graphics::{Color, Gradient, Style};

pub struct Laseretch;

#[derive(Clone, Debug)]
struct Spark {
    start: Coord,
    control: Coord,
    end: Coord,
    age: usize,
    lifetime: usize,
    symbol: &'static str,
}

impl Spark {
    fn position(&self) -> Coord {
        let progress = (self.age as f64 / self.lifetime.max(1) as f64)
            .clamp(0.0, 1.0);
        quadratic_bezier(
            self.start,
            self.control,
            self.end,
            easing::out_sine(progress),
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct EtchedCell {
    coord: Coord,
    symbol: char,
    color_index: usize,
}

impl Laseretch {
    pub fn new() -> Self {
        Self
    }

    fn style(color: Color, bold: bool) -> Style {
        Style {
            foreground: Some(color),
            bold,
            ..Style::default()
        }
    }

    fn render_frame(
        width: usize,
        height: usize,
        etched: &[EtchedCell],
        heat: &[usize],
        final_colors: &[Color],
        sparks: &[Spark],
        spark_colors: &[Color],
        laser_position: Option<Coord>,
        beam_colors: &[Color],
    ) -> String {
        let mut canvas = Canvas::new(width, height);
        let mut has_styled_cell = false;

        for (index, cell) in etched.iter().enumerate() {
            let final_color = final_colors
                .get(cell.color_index)
                .copied()
                .unwrap_or(Color::new(0, 212, 255));

            let color = match heat.get(index).copied().unwrap_or(usize::MAX) {
                0 => Color::new(255, 255, 255),
                1 => Color::new(255, 245, 170),
                2 => Color::new(255, 174, 66),
                3 => Color::new(255, 55, 32),
                4 => Color::new(255, 40, 90),
                _ => final_color,
            };

            canvas.set(
                cell.coord,
                cell.symbol.to_string(),
                Self::style(color, heat.get(index).copied().unwrap_or(5) < 3),
            );
            has_styled_cell = true;
        }

        for spark in sparks {
            let progress = spark.age.saturating_mul(spark_colors.len().max(1))
                / spark.lifetime.max(1);
            let color_index = progress.min(spark_colors.len().saturating_sub(1));
            let color = spark_colors
                .get(color_index)
                .copied()
                .unwrap_or(Color::new(255, 80, 20));

            if canvas.set(
                spark.position(),
                spark.symbol,
                Self::style(color, spark.age < spark.lifetime / 2),
            ) {
                has_styled_cell = true;
            }
        }

        if let Some(position) = laser_position {
            let last_row = position.row.max(0);

            for row in 0..=last_row {
                let color_index = if last_row == 0 {
                    beam_colors.len().saturating_sub(1)
                } else {
                    row as usize * beam_colors.len().saturating_sub(1)
                        / last_row as usize
                };
                let color = beam_colors
                    .get(color_index)
                    .copied()
                    .unwrap_or(Color::new(255, 0, 40));

                canvas.set(
                    Coord::new(position.column, row),
                    if row == last_row { "█" } else { "│" },
                    Self::style(color, true),
                );
                has_styled_cell = true;
            }
        }

        // Canvas emits SGR sequences only for styled cells. Keep blank input
        // visibly blank while still producing a styled effect frame.
        if !has_styled_cell {
            canvas.set(
                Coord::new(0, 0),
                " ",
                Self::style(Color::new(255, 0, 40), false),
            );
        }

        canvas.render()
    }

    fn next_random(state: &mut u64) -> u32 {
        let mut value = *state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        *state = value;
        (value >> 16) as u32
    }

    fn random_range(state: &mut u64, minimum: i32, maximum: i32) -> i32 {
        let span = maximum.saturating_sub(minimum).saturating_add(1) as u32;
        minimum + (Self::next_random(state) % span.max(1)) as i32
    }

    fn seed(input: &str) -> u64 {
        let mut hash = 0xcbf29ce484222325_u64;

        for byte in input.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }

        if hash == 0 {
            0x9e3779b97f4a7c15
        } else {
            hash
        }
    }

    fn add_sparks(
        sparks: &mut Vec<Spark>,
        position: Coord,
        canvas_bottom: i32,
        state: &mut u64,
    ) {
        const SYMBOLS: [&str; 4] = ["*", "·", ".", "+"];

        for _ in 0..3 {
            let end_column =
                position.column + Self::random_range(state, -20, 20);
            let control_row =
                position.row + Self::random_range(state, -10, 20);
            let lifetime = Self::random_range(state, 9, 16) as usize;
            let symbol =
                SYMBOLS[Self::next_random(state) as usize % SYMBOLS.len()];

            sparks.push(Spark {
                start: position,
                control: Coord::new(end_column, control_row),
                end: Coord::new(end_column, canvas_bottom),
                age: 0,
                lifetime,
                symbol,
            });
        }
    }

    fn advance(heat: &mut [usize], sparks: &mut Vec<Spark>) {
        for age in heat {
            *age = age.saturating_add(1);
        }

        for spark in sparks.iter_mut() {
            spark.age = spark.age.saturating_add(1);
        }

        sparks.retain(|spark| spark.age <= spark.lifetime);
    }
}

impl Default for Laseretch {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Laseretch {
    fn name(&self) -> &str {
        "laseretch"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let normalized = input.trim_end_matches(&['\r', '\n'][..]);
        let mut lines: Vec<Vec<char>> = if normalized.is_empty() {
            vec![Vec::new()]
        } else {
            normalized
                .split('\n')
                .map(|line| {
                    line.trim_end_matches('\r')
                        .chars()
                        .map(|character| {
                            if character == '\t' {
                                ' '
                            } else {
                                character
                            }
                        })
                        .collect()
                })
                .collect()
        };

        if lines.is_empty() {
            lines.push(Vec::new());
        }

        let width = lines.iter().map(Vec::len).max().unwrap_or(0).max(1);
        let height = lines.len().max(1);
        let cell_count = width.saturating_mul(height).max(2);

        let final_colors = Gradient::new(
            [
                Color::new(28, 48, 120),
                Color::new(0, 212, 255),
                Color::new(130, 255, 245),
                Color::new(255, 255, 255),
            ],
            cell_count,
        )
        .colors();

        let beam_colors = Gradient::new(
            [
                Color::new(120, 0, 24),
                Color::new(255, 0, 50),
                Color::new(255, 220, 220),
            ],
            height.max(2),
        )
        .colors();

        let spark_colors = Gradient::new(
            [
                Color::new(255, 255, 255),
                Color::new(255, 220, 80),
                Color::new(255, 80, 20),
                Color::new(110, 10, 25),
            ],
            16,
        )
        .colors();

        let mut targets = Vec::new();

        for (row, line) in lines.iter().enumerate() {
            let mut row_targets = line
                .iter()
                .enumerate()
                .filter_map(|(column, symbol)| {
                    if symbol.is_whitespace() {
                        None
                    } else {
                        Some(EtchedCell {
                            coord: Coord::new(column as i32, row as i32),
                            symbol: *symbol,
                            color_index: row * width + column,
                        })
                    }
                })
                .collect::<Vec<_>>();

            // The laser alternates direction per row instead of resetting
            // instantly to the left edge.
            if row % 2 == 1 {
                row_targets.reverse();
            }

            targets.extend(row_targets);
        }

        let mut frames = Vec::new();
        let mut etched = Vec::new();
        let mut heat = Vec::new();
        let mut sparks = Vec::new();
        let mut random_state = Self::seed(input);
        let canvas_bottom = height.saturating_sub(1) as i32;

        for target in targets {
            etched.push(target);
            heat.push(0);
            Self::add_sparks(
                &mut sparks,
                target.coord,
                canvas_bottom,
                &mut random_state,
            );

            // Hold briefly at each glyph so the white-hot etched character,
            // laser beam, and newly emitted sparks are visible.
            for _ in 0..2 {
                frames.push(Self::render_frame(
                    width,
                    height,
                    &etched,
                    &heat,
                    &final_colors,
                    &sparks,
                    &spark_colors,
                    Some(target.coord),
                    &beam_colors,
                ));
                Self::advance(&mut heat, &mut sparks);
            }
        }

        if etched.is_empty() {
            frames.push(Self::render_frame(
                width,
                height,
                &etched,
                &heat,
                &final_colors,
                &sparks,
                &spark_colors,
                None,
                &beam_colors,
            ));
            return frames;
        }

        // Let the sparks fall away and the freshly etched text cool into its
        // final coordinate-based gradient.
        let settling_frames = sparks
            .iter()
            .map(|spark| spark.lifetime.saturating_sub(spark.age))
            .max()
            .unwrap_or(0)
            .max(5);

        for _ in 0..settling_frames {
            frames.push(Self::render_frame(
                width,
                height,
                &etched,
                &heat,
                &final_colors,
                &sparks,
                &spark_colors,
                None,
                &beam_colors,
            ));
            Self::advance(&mut heat, &mut sparks);
        }

        heat.fill(usize::MAX);
        sparks.clear();
        frames.push(Self::render_frame(
            width,
            height,
            &etched,
            &heat,
            &final_colors,
            &sparks,
            &spark_colors,
            None,
            &beam_colors,
        ));

        frames
    }
}
