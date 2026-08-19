
use std::f64::consts::PI;

use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::{Color, Gradient, Style};
use crate::utils::geometry::Coord;

pub struct Bubbles;

impl Bubbles {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Bubbles {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Bubbles {
    fn name(&self) -> &str {
        "bubbles"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        let mut lines: Vec<&str> = normalized.split('\n').collect();

        if normalized.ends_with('\n') && lines.len() > 1 {
            lines.pop();
        }
        if lines.is_empty() {
            lines.push("");
        }

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let mut glyphs = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                glyphs.push(Glyph {
                    symbol: symbol.to_string(),
                    target: Coord::new(column as i32, row as i32),
                });
            }
        }

        let final_colors = Gradient::new(
            [
                Color::new(0x1f, 0x43, 0xff),
                Color::new(0x00, 0xd9, 0xff),
                Color::new(0xff, 0xff, 0xff),
            ],
            height,
        )
        .colors();

        if glyphs.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.set(
                Coord::new(0, 0),
                " ",
                colored_style(final_colors[0]),
            );
            return vec![canvas.render()];
        }

        let mut rng = SmallRng::new(hash_input(normalized.as_bytes()));
        let mut shuffled: Vec<usize> = (0..glyphs.len()).collect();
        shuffle(&mut shuffled, &mut rng);

        let bubble_colors = Gradient::new(
            [
                Color::new(0x00, 0xb8, 0xff),
                Color::new(0x65, 0x6d, 0xff),
                Color::new(0xd3, 0x3a, 0xff),
                Color::new(0xff, 0x5e, 0xb3),
            ],
            16,
        )
        .colors();

        let mut bubbles = Vec::new();
        let mut offset = 0usize;
        let mut bubble_index = 0usize;

        while offset < shuffled.len() {
            let remaining = shuffled.len() - offset;
            let desired = 5 + rng.range_usize(6);
            let group_size = desired.min(remaining);
            let members = shuffled[offset..offset + group_size].to_vec();
            offset += group_size;

            let radius = ((group_size as f64).sqrt() * 0.75)
                .round()
                .clamp(1.0, 3.0) as i32;
            let center_column = rng.range_usize(width) as i32;
            let destination_row = if height <= 2 {
                0
            } else {
                rng.range_usize((height / 2).max(1)) as i32
            };

            bubbles.push(Bubble {
                members,
                start_frame: bubble_index * 5,
                rise_frames: 18 + rng.range_usize(10),
                pop_frames: 5,
                settle_frames: 22 + rng.range_usize(8),
                radius,
                center_column,
                destination_row,
                angle_offset: rng.next_f64() * 2.0 * PI,
                color: bubble_colors[rng.range_usize(bubble_colors.len())],
            });

            bubble_index += 1;
        }

        let total_frames = bubbles
            .iter()
            .map(Bubble::completion_frame)
            .max()
            .unwrap_or(1);

        let mut frames = Vec::with_capacity(total_frames + 1);

        for frame_index in 0..=total_frames {
            let mut canvas = Canvas::new(width, height);

            for bubble in &bubbles {
                if frame_index < bubble.start_frame {
                    continue;
                }

                let local_frame = frame_index - bubble.start_frame;
                let member_count = bubble.members.len();

                for (member_index, &glyph_index) in bubble.members.iter().enumerate() {
                    let glyph = &glyphs[glyph_index];
                    let angle = bubble.angle_offset
                        + 2.0 * PI * member_index as f64 / member_count as f64;

                    let (position, symbol, color) = bubble_visual(
                        bubble,
                        glyph,
                        angle,
                        local_frame,
                        width,
                        height,
                        final_colors[glyph.target.row as usize],
                    );

                    canvas.set(position, symbol, colored_style(color));
                }
            }

            frames.push(canvas.render());
        }

        frames
    }
}

#[derive(Clone, Debug)]
struct Glyph {
    symbol: String,
    target: Coord,
}

#[derive(Clone, Debug)]
struct Bubble {
    members: Vec<usize>,
    start_frame: usize,
    rise_frames: usize,
    pop_frames: usize,
    settle_frames: usize,
    radius: i32,
    center_column: i32,
    destination_row: i32,
    angle_offset: f64,
    color: Color,
}

impl Bubble {
    fn completion_frame(&self) -> usize {
        self.start_frame + self.rise_frames + self.pop_frames + self.settle_frames
    }

    fn circle_position(&self, angle: f64, center_row: f64, expansion: f64) -> Coord {
        let radius = self.radius as f64 + expansion;
        Coord::new(
            (self.center_column as f64 + angle.cos() * radius).round() as i32,
            (center_row + angle.sin() * radius * 0.55).round() as i32,
        )
    }
}

fn bubble_visual(
    bubble: &Bubble,
    glyph: &Glyph,
    angle: f64,
    local_frame: usize,
    width: usize,
    height: usize,
    final_color: Color,
) -> (Coord, String, Color) {
    let rise_end = bubble.rise_frames;
    let pop_end = rise_end + bubble.pop_frames;
    let settle_end = pop_end + bubble.settle_frames;

    let bottom_row = height.saturating_sub(1) as f64;

    if local_frame < rise_end {
        let progress = local_frame as f64 / bubble.rise_frames.max(1) as f64;
        let eased = easing::out_sine(progress);
        let center_row = bottom_row
            + (bubble.destination_row as f64 - bottom_row) * eased;
        let position = bubble.circle_position(angle, center_row, 0.0);

        return (
            clamp_coord(position, width, height),
            glyph.symbol.clone(),
            bubble.color,
        );
    }

    if local_frame < pop_end {
        let progress =
            (local_frame - rise_end) as f64 / bubble.pop_frames.max(1) as f64;
        let eased = easing::out_cubic(progress);
        let position = bubble.circle_position(
            angle,
            bubble.destination_row as f64,
            3.0 * eased,
        );
        let symbol = if progress < 0.34 {
            "o"
        } else if progress < 0.72 {
            "*"
        } else {
            "."
        };

        return (
            clamp_coord(position, width, height),
            symbol.to_owned(),
            bubble.color.lerp(Color::new(0xff, 0xff, 0xff), eased),
        );
    }

    if local_frame < settle_end {
        let progress =
            (local_frame - pop_end) as f64 / bubble.settle_frames.max(1) as f64;
        let eased = easing::out_bounce(progress);
        let expanded = bubble.circle_position(
            angle,
            bubble.destination_row as f64,
            3.0,
        );
        let position = interpolate_coord(expanded, glyph.target, eased);

        return (
            clamp_coord(position, width, height),
            glyph.symbol.clone(),
            Color::new(0xff, 0xff, 0xff).lerp(final_color, eased),
        );
    }

    (glyph.target, glyph.symbol.clone(), final_color)
}

fn interpolate_coord(start: Coord, end: Coord, progress: f64) -> Coord {
    let progress = progress.clamp(0.0, 1.0);
    let column = start.column as f64
        + (end.column - start.column) as f64 * progress;
    let row = start.row as f64 + (end.row - start.row) as f64 * progress;

    Coord::new(column.round() as i32, row.round() as i32)
}

fn clamp_coord(coord: Coord, width: usize, height: usize) -> Coord {
    Coord::new(
        coord.column.clamp(0, width.saturating_sub(1) as i32),
        coord.row.clamp(0, height.saturating_sub(1) as i32),
    )
}

fn colored_style(color: Color) -> Style {
    Style {
        foreground: Some(color),
        ..Style::default()
    }
}

fn hash_input(input: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;

    for &byte in input {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    if hash == 0 {
        0x9e37_79b9_7f4a_7c15
    } else {
        hash
    }
}

fn shuffle(values: &mut [usize], rng: &mut SmallRng) {
    for index in (1..values.len()).rev() {
        let other = rng.range_usize(index + 1);
        values.swap(index, other);
    }
}

#[derive(Clone, Debug)]
struct SmallRng {
    state: u64,
}

impl SmallRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
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

    fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    fn range_usize(&mut self, upper_bound: usize) -> usize {
        if upper_bound <= 1 {
            0
        } else {
            (self.next_u64() % upper_bound as u64) as usize
        }
    }
}
