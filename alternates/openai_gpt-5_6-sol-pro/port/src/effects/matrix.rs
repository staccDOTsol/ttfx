
use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

pub struct Matrix;

impl Matrix {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Matrix {
    fn name(&self) -> &str {
        "matrix"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let target = TargetText::from_input(input);
        let seed = hash_bytes(input.as_bytes());
        let mut simulation =
            MatrixSimulation::new(target.width, target.height, seed);

        let rain_frames = (24 + target.height * 3 + target.width / 2)
            .clamp(24, 120);
        let resolve_frames = (20 + target.height * 2).clamp(20, 100);
        let mut frames =
            Vec::with_capacity(rain_frames + resolve_frames + 3);
        let mut resolved = vec![false; target.width * target.height];

        for frame_index in 0..rain_frames {
            simulation.advance(frame_index);
            frames.push(simulation.render(
                &target,
                &resolved,
                frame_index,
            ));
        }

        let mut resolution_order =
            (0..resolved.len()).collect::<Vec<usize>>();
        shuffle(&mut resolution_order, &mut simulation.rng);

        let cells_per_frame =
            resolution_order.len().div_ceil(resolve_frames).max(1);
        let mut resolved_count = 0;

        for frame_offset in 0..resolve_frames {
            let frame_index = rain_frames + frame_offset;
            simulation.advance(frame_index);

            let next_count = (resolved_count + cells_per_frame)
                .min(resolution_order.len());

            for &index in &resolution_order[resolved_count..next_count] {
                resolved[index] = true;
            }
            resolved_count = next_count;

            frames.push(simulation.render(
                &target,
                &resolved,
                frame_index,
            ));

            if resolved_count == resolution_order.len() {
                break;
            }
        }

        resolved.fill(true);
        let final_frame =
            simulation.render(&target, &resolved, rain_frames + resolve_frames);

        for _ in 0..3 {
            frames.push(final_frame.clone());
        }

        frames
    }
}

struct TargetText {
    width: usize,
    height: usize,
    cells: Vec<char>,
    colors: Vec<Color>,
}

impl TargetText {
    fn from_input(input: &str) -> Self {
        let trimmed =
            input.trim_end_matches(|character| character == '\n' || character == '\r');

        let lines = if trimmed.is_empty() {
            vec![Vec::new()]
        } else {
            trimmed
                .split('\n')
                .map(|line| {
                    line.trim_end_matches('\r').chars().collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };

        let width = lines
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);
        let mut cells = vec![' '; width * height];

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.iter().copied().enumerate() {
                cells[row * width + column] = symbol;
            }
        }

        let row_colors = Gradient::new(
            [
                Color::new(215, 255, 224),
                Color::new(90, 255, 135),
                Color::new(0, 255, 65),
            ],
            height,
        )
        .colors();

        let mut colors = Vec::with_capacity(width * height);
        for row in 0..height {
            let row_color = row_colors[row];
            for column in 0..width {
                let horizontal_progress = if width <= 1 {
                    1.0
                } else {
                    column as f64 / (width - 1) as f64
                };
                let center_emphasis =
                    1.0 - (horizontal_progress * 2.0 - 1.0).abs();
                colors.push(
                    row_color.lerp(
                        Color::new(235, 255, 238),
                        center_emphasis * 0.22,
                    ),
                );
            }
        }

        Self {
            width,
            height,
            cells,
            colors,
        }
    }
}

struct Stream {
    head: i32,
    tail_length: usize,
    period: usize,
    phase: usize,
    symbol_seed: u64,
}

struct MatrixSimulation {
    width: usize,
    height: usize,
    streams: Vec<Stream>,
    rain_colors: Vec<Color>,
    rng: MatrixRng,
}

impl MatrixSimulation {
    fn new(width: usize, height: usize, seed: u64) -> Self {
        let mut rng = MatrixRng::new(seed);
        let mut streams = Vec::with_capacity(width);

        for column in 0..width {
            let delay_limit = height.saturating_mul(2).saturating_add(8);
            streams.push(Stream {
                head: -(rng.range(delay_limit.max(1)) as i32),
                tail_length: 4 + rng.range(height.clamp(4, 18)),
                period: 1 + rng.range(2),
                phase: rng.range(3),
                symbol_seed: rng.next_u64() ^ column as u64,
            });
        }

        let rain_colors = Gradient::new(
            [
                Color::new(225, 255, 230),
                Color::new(90, 255, 120),
                Color::new(0, 210, 55),
                Color::new(0, 90, 25),
                Color::new(0, 24, 8),
            ],
            20,
        )
        .colors();

        Self {
            width,
            height,
            streams,
            rain_colors,
            rng,
        }
    }

    fn advance(&mut self, frame_index: usize) {
        for stream_index in 0..self.streams.len() {
            let should_advance = {
                let stream = &self.streams[stream_index];
                (frame_index + stream.phase) % stream.period == 0
            };

            if !should_advance {
                continue;
            }

            self.streams[stream_index].head += 1;

            let finished = {
                let stream = &self.streams[stream_index];
                stream.head - stream.tail_length as i32
                    > self.height as i32
            };

            if finished {
                let delay = self.rng.range(
                    self.height.saturating_mul(2).saturating_add(8),
                );
                let stream = &mut self.streams[stream_index];
                stream.head = -(delay as i32);
                stream.tail_length =
                    4 + self.rng.range(self.height.clamp(4, 18));
                stream.period = 1 + self.rng.range(2);
                stream.phase = self.rng.range(3);
                stream.symbol_seed = self.rng.next_u64();
            }
        }
    }

    fn render(
        &self,
        target: &TargetText,
        resolved: &[bool],
        frame_index: usize,
    ) -> String {
        let mut canvas = Canvas::new(self.width, self.height);
        canvas.fill(
            " ",
            Style {
                foreground: Some(Color::new(0, 18, 5)),
                ..Style::default()
            },
        );

        for (column, stream) in self.streams.iter().enumerate() {
            for row in 0..self.height {
                let distance = stream.head - row as i32;
                if distance < 0 || distance as usize >= stream.tail_length {
                    continue;
                }

                let distance = distance as usize;
                let color_index = if stream.tail_length <= 1 {
                    0
                } else {
                    distance
                        .saturating_mul(self.rain_colors.len() - 1)
                        / (stream.tail_length - 1)
                };
                let color = self.rain_colors[color_index];
                let symbol = matrix_symbol(
                    stream.symbol_seed,
                    column,
                    row,
                    frame_index,
                );

                canvas.set(
                    Coord::new(column as i32, row as i32),
                    symbol.to_string(),
                    Style {
                        foreground: Some(color),
                        bold: distance == 0,
                        ..Style::default()
                    },
                );
            }
        }

        for row in 0..target.height {
            for column in 0..target.width {
                let index = row * target.width + column;
                if !resolved[index] {
                    continue;
                }

                canvas.set(
                    Coord::new(column as i32, row as i32),
                    target.cells[index].to_string(),
                    Style {
                        foreground: Some(target.colors[index]),
                        bold: true,
                        ..Style::default()
                    },
                );
            }
        }

        canvas.render()
    }
}

fn matrix_symbol(
    seed: u64,
    column: usize,
    row: usize,
    frame_index: usize,
) -> char {
    const SYMBOLS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut value = seed
        ^ (column as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (row as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
        ^ ((frame_index / 3) as u64).wrapping_mul(0x94d0_49bb_1331_11eb);

    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;

    SYMBOLS[value as usize % SYMBOLS.len()] as char
}

fn shuffle(values: &mut [usize], rng: &mut MatrixRng) {
    for index in (1..values.len()).rev() {
        let other = rng.range(index + 1);
        values.swap(index, other);
    }
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash ^ 0x4d41_5452_4958
}

struct MatrixRng {
    state: u64,
}

impl MatrixRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x6a09_e667_f3bc_c909
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
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
            (self.next_u64() % upper as u64) as usize
        }
    }
}
