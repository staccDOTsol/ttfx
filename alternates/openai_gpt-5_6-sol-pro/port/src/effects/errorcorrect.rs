use super::Effect;

use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, Gradient, Style};

const ERROR_COLOR: Color = Color::new(255, 0, 0);
const CORRECT_COLOR: Color = Color::new(0, 255, 0);
const MOVEMENT_SPEED: f64 = 0.5;
const SWAP_DELAY: usize = 10;
const ERROR_PAIRS: usize = 3;
const COLOR_FADE_FRAMES: usize = 8;

#[derive(Clone, Debug)]
struct Glyph {
    symbol: String,
    input_coord: Coord,
    final_color: Color,
}

#[derive(Clone, Copy, Debug)]
struct ErrorPair {
    first: usize,
    second: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Errorcorrect;

impl Errorcorrect {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Errorcorrect {
    fn name(&self) -> &str {
        "errorcorrect"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let (width, height, mut glyphs) = parse_input(input);
        assign_final_colors(&mut glyphs, height);

        if glyphs.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style {
                    foreground: Some(Color::new(138, 0, 138)),
                    ..Style::default()
                },
            );
            return vec![canvas.render()];
        }

        let final_positions = glyphs
            .iter()
            .map(|glyph| glyph.input_coord)
            .collect::<Vec<_>>();
        let final_colors = glyphs
            .iter()
            .map(|glyph| glyph.final_color)
            .collect::<Vec<_>>();

        let pairs = select_error_pairs(&glyphs, input);
        let mut frames = vec![render_frame(
            width,
            height,
            &glyphs,
            &final_positions,
            &final_colors,
        )];

        if pairs.is_empty() {
            return frames;
        }

        let mut swapped_positions = final_positions.clone();
        for pair in &pairs {
            swapped_positions[pair.first] = final_positions[pair.second];
            swapped_positions[pair.second] = final_positions[pair.first];
        }

        let error_colors =
            colors_for_pairs(&final_colors, &pairs, ERROR_COLOR);
        let correct_colors =
            colors_for_pairs(&final_colors, &pairs, CORRECT_COLOR);

        let movement_frames =
            movement_frame_count(&final_positions, &swapped_positions);

        for frame_index in 1..=movement_frames {
            let progress =
                easing::in_out_sine(frame_index as f64 / movement_frames as f64);
            let positions = interpolate_positions(
                &final_positions,
                &swapped_positions,
                progress,
            );

            frames.push(render_frame(
                width,
                height,
                &glyphs,
                &positions,
                &error_colors,
            ));
        }

        for _ in 0..SWAP_DELAY {
            frames.push(render_frame(
                width,
                height,
                &glyphs,
                &swapped_positions,
                &error_colors,
            ));
        }

        for frame_index in 1..=movement_frames {
            let progress =
                easing::in_out_sine(frame_index as f64 / movement_frames as f64);
            let positions = interpolate_positions(
                &swapped_positions,
                &final_positions,
                progress,
            );

            frames.push(render_frame(
                width,
                height,
                &glyphs,
                &positions,
                &correct_colors,
            ));
        }

        for frame_index in 1..=COLOR_FADE_FRAMES {
            let progress =
                frame_index as f64 / COLOR_FADE_FRAMES as f64;
            let mut colors = final_colors.clone();

            for pair in &pairs {
                colors[pair.first] =
                    CORRECT_COLOR.lerp(final_colors[pair.first], progress);
                colors[pair.second] =
                    CORRECT_COLOR.lerp(final_colors[pair.second], progress);
            }

            frames.push(render_frame(
                width,
                height,
                &glyphs,
                &final_positions,
                &colors,
            ));
        }

        frames
    }
}

fn parse_input(input: &str) -> (usize, usize, Vec<Glyph>) {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized.split('\n').collect::<Vec<_>>();

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
        .unwrap_or(1)
        .max(1);
    let height = lines.len().max(1);

    let mut glyphs = Vec::new();

    for (row, line) in lines.iter().enumerate() {
        for (column, symbol) in line.chars().enumerate() {
            glyphs.push(Glyph {
                symbol: symbol.to_string(),
                input_coord: Coord::new(column as i32, row as i32),
                final_color: Color::default(),
            });
        }
    }

    (width, height, glyphs)
}

fn assign_final_colors(glyphs: &mut [Glyph], height: usize) {
    let gradient = Gradient::new(
        [
            Color::new(138, 0, 138),
            Color::new(0, 209, 255),
            Color::new(255, 255, 255),
        ],
        height.max(2),
    );
    let colors = gradient.colors();

    for glyph in glyphs {
        let row = glyph.input_coord.row.max(0) as usize;
        glyph.final_color = colors
            .get(row)
            .copied()
            .or_else(|| colors.last().copied())
            .unwrap_or(Color::new(255, 255, 255));
    }
}

fn select_error_pairs(glyphs: &[Glyph], input: &str) -> Vec<ErrorPair> {
    let mut indices = glyphs
        .iter()
        .enumerate()
        .filter_map(|(index, glyph)| {
            if glyph.symbol.chars().all(char::is_whitespace) {
                None
            } else {
                Some(index)
            }
        })
        .collect::<Vec<_>>();

    if indices.len() < 2 {
        return Vec::new();
    }

    let mut state = input.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
    });

    for index in (1..indices.len()).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let selected = (state as usize) % (index + 1);
        indices.swap(index, selected);
    }

    let pair_count = ERROR_PAIRS.min(indices.len() / 2);
    let mut pairs = Vec::with_capacity(pair_count);

    for pair_index in 0..pair_count {
        let first_offset = pair_index * 2;
        let mut second_offset = first_offset + 1;

        if glyphs[indices[first_offset]].symbol
            == glyphs[indices[second_offset]].symbol
        {
            if let Some(offset) =
                ((second_offset + 1)..indices.len()).find(|offset| {
                    glyphs[indices[*offset]].symbol
                        != glyphs[indices[first_offset]].symbol
                })
            {
                indices.swap(second_offset, offset);
                second_offset = first_offset + 1;
            }
        }

        pairs.push(ErrorPair {
            first: indices[first_offset],
            second: indices[second_offset],
        });
    }

    pairs
}

fn movement_frame_count(start: &[Coord], end: &[Coord]) -> usize {
    let maximum_distance = start
        .iter()
        .zip(end)
        .map(|(start, end)| start.distance_to(*end))
        .fold(0.0_f64, f64::max);

    (maximum_distance / MOVEMENT_SPEED).ceil().max(1.0) as usize
}

fn interpolate_positions(
    start: &[Coord],
    end: &[Coord],
    progress: f64,
) -> Vec<Coord> {
    start
        .iter()
        .zip(end)
        .map(|(start, end)| start.lerp(*end, progress))
        .collect()
}

fn colors_for_pairs(
    final_colors: &[Color],
    pairs: &[ErrorPair],
    pair_color: Color,
) -> Vec<Color> {
    let mut colors = final_colors.to_vec();

    for pair in pairs {
        colors[pair.first] = pair_color;
        colors[pair.second] = pair_color;
    }

    colors
}

fn render_frame(
    width: usize,
    height: usize,
    glyphs: &[Glyph],
    positions: &[Coord],
    colors: &[Color],
) -> String {
    let mut canvas = Canvas::new(width, height);

    for ((glyph, position), color) in
        glyphs.iter().zip(positions).zip(colors)
    {
        canvas.set(
            *position,
            glyph.symbol.clone(),
            Style {
                foreground: Some(*color),
                ..Style::default()
            },
        );
    }

    canvas.render()
}
