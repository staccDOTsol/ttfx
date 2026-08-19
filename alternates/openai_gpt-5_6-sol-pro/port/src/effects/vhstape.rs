
use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

pub struct Vhstape;

impl Vhstape {
    pub fn new() -> Self {
        Self
    }

    fn input_rows(input: &str) -> Vec<Vec<String>> {
        let mut rows = input
            .lines()
            .map(|line| {
                line.trim_end_matches('\r')
                    .chars()
                    .map(|character| character.to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        if rows.is_empty() {
            rows.push(Vec::new());
        }

        rows
    }

    fn final_colors(height: usize) -> Vec<Color> {
        Gradient::new(
            [
                Color::new(0xab, 0x48, 0xff),
                Color::new(0xe7, 0xb2, 0xb2),
                Color::new(0xff, 0xfe, 0xbd),
            ],
            height,
        )
        .colors()
    }

    fn scaled_color(color: Color, scale: f64) -> Color {
        let scale = scale.clamp(0.0, 1.5);
        Color::new(
            (color.red as f64 * scale).round().clamp(0.0, 255.0) as u8,
            (color.green as f64 * scale)
                .round()
                .clamp(0.0, 255.0) as u8,
            (color.blue as f64 * scale)
                .round()
                .clamp(0.0, 255.0) as u8,
        )
    }

    fn style(color: Color, bold: bool) -> Style {
        Style {
            foreground: Some(color),
            bold,
            ..Style::default()
        }
    }

    fn render_clean(
        rows: &[Vec<String>],
        width: usize,
        height: usize,
        colors: &[Color],
        brightness: f64,
    ) -> String {
        let mut canvas = Canvas::new(width, height);

        for row in 0..height {
            let color = Self::scaled_color(colors[row], brightness);

            for column in 0..width {
                let symbol = rows[row]
                    .get(column)
                    .cloned()
                    .unwrap_or_else(|| " ".to_owned());

                canvas.set(
                    Coord::new(column as i32, row as i32),
                    symbol,
                    Self::style(color, false),
                );
            }
        }

        canvas.render()
    }

    fn render_glitch(
        rows: &[Vec<String>],
        width: usize,
        height: usize,
        colors: &[Color],
        intensity: f64,
        frame_index: usize,
        rng: &mut VhsRng,
    ) -> String {
        const STATIC_SYMBOLS: [&str; 8] =
            ["░", "▒", "▓", "█", "▌", "▐", "╱", "╲"];
        const GLITCH_COLORS: [Color; 5] = [
            Color::new(0xff, 0xff, 0xff),
            Color::new(0xff, 0x24, 0x4f),
            Color::new(0x25, 0xff, 0xb0),
            Color::new(0x36, 0x74, 0xff),
            Color::new(0xd8, 0x58, 0xff),
        ];

        let intensity = intensity.clamp(0.0, 1.0);
        let maximum_shift = ((width / 4).clamp(1, 8) as f64 * intensity)
            .ceil()
            .max(1.0) as i32;
        let tracking_row = if height > 1 && rng.chance(0.22 + intensity * 0.42)
        {
            Some(rng.range_usize(0, height))
        } else {
            None
        };
        let tracking_thickness =
            1 + usize::from(intensity > 0.72 && rng.chance(0.4));

        let mut row_shifts = vec![0_i32; height];
        for (row, shift) in row_shifts.iter_mut().enumerate() {
            let wave = (((frame_index as f64 * 0.71) + row as f64 * 0.83)
                .sin()
                * maximum_shift as f64
                * intensity
                * 0.45)
                .round() as i32;

            if rng.chance(0.08 + intensity * 0.34) {
                *shift =
                    rng.range_i32(-maximum_shift, maximum_shift + 1) + wave;
            }
        }

        let mut canvas = Canvas::new(width, height);

        for row in 0..height {
            let in_tracking_band = tracking_row.is_some_and(|tracking| {
                row >= tracking && row < tracking.saturating_add(tracking_thickness)
            });
            let scanline_scale = if row % 2 == 0 { 0.82 } else { 1.0 };
            let shift = row_shifts[row];

            for column in 0..width {
                let source_column = column as i32 - shift;
                let mut symbol = if source_column >= 0 {
                    rows[row]
                        .get(source_column as usize)
                        .cloned()
                        .unwrap_or_else(|| " ".to_owned())
                } else {
                    " ".to_owned()
                };

                let edge_noise = shift != 0
                    && (source_column < 0 || source_column >= width as i32);
                let static_chance = if in_tracking_band {
                    0.45 + intensity * 0.35
                } else {
                    0.015 + intensity * 0.12
                };

                if edge_noise || rng.chance(static_chance) {
                    symbol = STATIC_SYMBOLS
                        [rng.range_usize(0, STATIC_SYMBOLS.len())]
                    .to_owned();
                }

                let mut color = colors[row];

                if shift != 0 && rng.chance(0.58 + intensity * 0.3) {
                    let color_index =
                        (column + row + frame_index) % GLITCH_COLORS.len();
                    color = GLITCH_COLORS[color_index];
                } else if in_tracking_band {
                    color = GLITCH_COLORS
                        [rng.range_usize(0, GLITCH_COLORS.len())];
                } else if rng.chance(0.02 + intensity * 0.08) {
                    color = GLITCH_COLORS
                        [rng.range_usize(0, GLITCH_COLORS.len())];
                }

                let flicker =
                    0.88 + rng.next_f64() * 0.18 * intensity.max(0.15);
                color =
                    Self::scaled_color(color, scanline_scale * flicker);

                canvas.set(
                    Coord::new(column as i32, row as i32),
                    symbol,
                    Self::style(color, in_tracking_band),
                );
            }
        }

        canvas.render()
    }
}

impl Default for Vhstape {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Vhstape {
    fn name(&self) -> &str {
        "vhstape"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let rows = Self::input_rows(input);
        let width = rows
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(1);
        let height = rows.len().max(1);
        let colors = Self::final_colors(height);
        let mut rng = VhsRng::new(input);
        let mut frames = Vec::new();

        // Briefly display the recording before the tracking distortion begins.
        frames.push(Self::render_clean(
            &rows, width, height, &colors, 0.55,
        ));
        frames.push(Self::render_clean(
            &rows, width, height, &colors, 0.72,
        ));

        // Main VHS tracking phase: horizontal displacement, chromatic
        // separation, scanlines, and intermittent static.
        for frame_index in 0..30 {
            let pulse =
                ((frame_index as f64 * 0.61).sin().abs() * 0.36) + 0.48;
            let burst = if frame_index % 9 == 5 || frame_index % 13 == 8 {
                0.28
            } else {
                0.0
            };

            frames.push(Self::render_glitch(
                &rows,
                width,
                height,
                &colors,
                (pulse + burst).min(1.0),
                frame_index,
                &mut rng,
            ));
        }

        // Let the tracking settle while retaining progressively weaker noise.
        for settle_index in 0..10 {
            let intensity = 0.42 * (1.0 - settle_index as f64 / 10.0);
            frames.push(Self::render_glitch(
                &rows,
                width,
                height,
                &colors,
                intensity,
                30 + settle_index,
                &mut rng,
            ));
        }

        frames.push(Self::render_clean(
            &rows, width, height, &colors, 0.88,
        ));
        frames.push(Self::render_clean(
            &rows, width, height, &colors, 1.0,
        ));

        frames
    }
}

struct VhsRng {
    state: u64,
}

impl VhsRng {
    fn new(input: &str) -> Self {
        let mut state = 0xcbf2_9ce4_8422_2325_u64;

        for byte in input.bytes() {
            state ^= u64::from(byte);
            state = state.wrapping_mul(0x0000_0100_0000_01b3);
        }

        if state == 0 {
            state = 0x9e37_79b9_7f4a_7c15;
        }

        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state = self
            .state
            .wrapping_mul(0x2545_f491_4f6c_dd1d);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        value as f64 / ((1_u64 << 53) as f64)
    }

    fn chance(&mut self, probability: f64) -> bool {
        self.next_f64() < probability.clamp(0.0, 1.0)
    }

    fn range_usize(&mut self, start: usize, end: usize) -> usize {
        if end <= start {
            return start;
        }

        start + (self.next_u64() as usize % (end - start))
    }

    fn range_i32(&mut self, start: i32, end: i32) -> i32 {
        if end <= start {
            return start;
        }

        start + (self.next_u64() % (end - start) as u64) as i32
    }
}
