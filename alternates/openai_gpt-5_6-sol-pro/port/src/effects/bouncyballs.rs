
use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::{Color, Coord, Gradient, Style};

pub struct Bouncyballs;

impl Bouncyballs {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Bouncyballs {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Bouncyballs {
    fn name(&self) -> &str {
        "bouncyballs"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let final_colors = Gradient::new(
            [
                Color::new(0x31, 0xa0, 0xd4),
                Color::new(0x5a, 0xcd, 0xa9),
                Color::new(0x8a, 0x5b, 0xac),
            ],
            24,
        )
        .colors();

        let ball_colors = Gradient::new(
            [
                Color::new(0xd1, 0xf4, 0xa5),
                Color::new(0x96, 0xe2, 0xa4),
                Color::new(0x5a, 0xcd, 0xa9),
                Color::new(0x00, 0xb8, 0xb8),
                Color::new(0x00, 0x81, 0x8a),
            ],
            24,
        )
        .colors();

        let mut balls = Vec::new();

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let denominator = width.saturating_sub(1) + height.saturating_sub(1);
                let gradient_progress = if denominator == 0 {
                    0.0
                } else {
                    (column + row) as f64 / denominator as f64
                };
                let color_index = (gradient_progress
                    * final_colors.len().saturating_sub(1) as f64)
                    .round() as usize;

                balls.push(Ball {
                    symbol: symbol.to_string(),
                    target: Coord::new(column as i32, row as i32),
                    launch_frame: 0,
                    duration: (12 + row * 4).max(12),
                    ball_color: ball_colors[0],
                    final_color: final_colors[color_index.min(final_colors.len() - 1)],
                });
            }
        }

        if balls.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style {
                    foreground: Some(final_colors[0]),
                    ..Style::default()
                },
            );
            return vec![canvas.render()];
        }

        let mut random = DeterministicRandom::new(hash_input(input));
        let mut order: Vec<usize> = (0..balls.len()).collect();

        for index in (1..order.len()).rev() {
            let other = random.range(index + 1);
            order.swap(index, other);
        }

        let mut cursor = 0;
        let mut launch_frame = 0;

        while cursor < order.len() {
            let group_size = (2 + random.range(5)).min(order.len() - cursor);

            for &ball_index in &order[cursor..cursor + group_size] {
                balls[ball_index].launch_frame = launch_frame;
                balls[ball_index].ball_color =
                    ball_colors[random.range(ball_colors.len())];
            }

            cursor += group_size;
            launch_frame += 3;
        }

        let final_frame = balls
            .iter()
            .map(|ball| ball.launch_frame + ball.duration)
            .max()
            .unwrap_or(0);

        let mut frames = Vec::with_capacity(final_frame + 3);

        for frame_number in 0..=final_frame + 2 {
            let mut canvas = Canvas::new(width, height);

            for ball in &balls {
                if frame_number < ball.launch_frame {
                    continue;
                }

                let elapsed = frame_number - ball.launch_frame;
                let settled = elapsed >= ball.duration;
                let progress = if settled {
                    1.0
                } else {
                    elapsed as f64 / ball.duration as f64
                };
                let bounced_progress = easing::out_bounce(progress);

                let row = if settled {
                    ball.target.row
                } else {
                    (ball.target.row as f64 * bounced_progress).round() as i32
                };

                let color = if settled {
                    ball.final_color
                } else {
                    ball.ball_color
                };

                canvas.set(
                    Coord::new(ball.target.column, row),
                    ball.symbol.clone(),
                    Style {
                        foreground: Some(color),
                        bold: !settled,
                        ..Style::default()
                    },
                );
            }

            frames.push(canvas.render());
        }

        frames
    }
}

struct Ball {
    symbol: String,
    target: Coord,
    launch_frame: usize,
    duration: usize,
    ball_color: Color,
    final_color: Color,
}

struct DeterministicRandom {
    state: u64,
}

impl DeterministicRandom {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
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
}

fn hash_input(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash
}
