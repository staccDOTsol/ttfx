use super::Effect;

use crate::engine::{Canvas, CharacterId, CharacterVisual, EffectCharacter};
use crate::utils::easing;
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

const UNSTABLE_COLOR: Color = Color::new(255, 255, 255);
const FINAL_GRADIENT_START: Color = Color::new(138, 0, 138);
const FINAL_GRADIENT_MIDDLE: Color = Color::new(0, 209, 255);
const FINAL_GRADIENT_END: Color = Color::new(255, 255, 255);

#[derive(Clone, Debug)]
struct Glyph {
    id: CharacterId,
    symbol: String,
    origin: Coord,
    scattered: Coord,
    final_color: Color,
}

pub struct Unstable;

impl Unstable {
    pub fn new() -> Self {
        Self
    }

    fn parse_input(input: &str) -> (usize, usize, Vec<(String, Coord)>) {
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        let normalized = normalized.trim_end_matches('\n');
        let lines: Vec<&str> = if normalized.is_empty() {
            vec![""]
        } else {
            normalized.split('\n').collect()
        };

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let mut parsed = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                parsed.push((
                    symbol.to_string(),
                    Coord::new(column as i32, row as i32),
                ));
            }
        }

        (width, height, parsed)
    }

    fn build_glyphs(
        width: usize,
        height: usize,
        parsed: Vec<(String, Coord)>,
    ) -> Vec<Glyph> {
        let gradient_steps = height.max(12);
        let gradient = Gradient::new(
            [
                FINAL_GRADIENT_START,
                FINAL_GRADIENT_MIDDLE,
                FINAL_GRADIENT_END,
            ],
            gradient_steps,
        )
        .colors();

        parsed
            .into_iter()
            .enumerate()
            .map(|(index, (symbol, origin))| {
                let seed = index as u64 + 1;
                let scattered = Coord::new(
                    Self::random_coordinate(seed, 0, width),
                    Self::random_coordinate(seed, 1, height),
                );

                let color_index = if height <= 1 {
                    index % gradient.len()
                } else {
                    let progress = origin.row as f64 / (height - 1) as f64;
                    (progress * (gradient.len() - 1) as f64).round() as usize
                };

                Glyph {
                    id: CharacterId(index as u32),
                    symbol,
                    origin,
                    scattered,
                    final_color: gradient[color_index.min(gradient.len() - 1)],
                }
            })
            .collect()
    }

    fn random_coordinate(
        seed: u64,
        axis: u64,
        extent: usize,
    ) -> i32 {
        if extent <= 1 {
            return 0;
        }

        (Self::mix(seed ^ axis.wrapping_mul(0x9e37_79b9_7f4a_7c15))
            % extent as u64) as i32
    }

    fn mix(mut value: u64) -> u64 {
        value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        value = (value ^ (value >> 30))
            .wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27))
            .wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn jitter(
        id: CharacterId,
        frame: usize,
        amplitude: i32,
    ) -> Coord {
        if amplitude <= 0 {
            return Coord::new(0, 0);
        }

        let span = (amplitude * 2 + 1) as u64;
        let base = id.0 as u64
            ^ (frame as u64).wrapping_mul(0xd6e8_feb8_6659_fd93);

        let column =
            (Self::mix(base ^ 0xa076_1d64_78bd_642f) % span) as i32
                - amplitude;
        let row =
            (Self::mix(base ^ 0xe703_7ed1_a0b4_28db) % span) as i32
                - amplitude;

        Coord::new(column, row)
    }

    fn render(
        width: usize,
        height: usize,
        glyphs: &[Glyph],
        frame_number: usize,
        position: impl Fn(&Glyph) -> Coord,
        color_progress: f64,
        jitter_amplitude: i32,
    ) -> String {
        let mut canvas = Canvas::new(width, height);

        if glyphs.is_empty() {
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style::with_colors(ColorPair::new(
                    Some(UNSTABLE_COLOR),
                    None,
                )),
            );
            return canvas.render();
        }

        for glyph in glyphs {
            let base_position = position(glyph);
            let offset =
                Self::jitter(glyph.id, frame_number, jitter_amplitude);
            let current_position = Coord::new(
                base_position.column + offset.column,
                base_position.row + offset.row,
            );

            let color =
                UNSTABLE_COLOR.lerp(glyph.final_color, color_progress);
            let style = Style {
                bold: color_progress < 0.8,
                ..Style::with_colors(ColorPair::new(Some(color), None))
            };

            let mut character = EffectCharacter::new(
                glyph.id,
                glyph.symbol.clone(),
                current_position,
            );
            character.set_appearance(CharacterVisual::new(
                glyph.symbol.clone(),
                style,
            ));
            canvas.draw_character(&character);
        }

        let rendered = canvas.render();
        if rendered.contains('\x1b') {
            rendered
        } else {
            let style = Style::with_colors(ColorPair::new(
                Some(UNSTABLE_COLOR),
                None,
            ));
            format!("{}{}\x1b[0m", style.ansi_prefix(), rendered)
        }
    }
}

impl Default for Unstable {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Unstable {
    fn name(&self) -> &str {
        "unstable"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let (width, height, parsed) = Self::parse_input(input);
        let glyphs = Self::build_glyphs(width, height, parsed);
        let mut frames = Vec::new();
        let mut frame_number = 0;

        const DESTABILIZE_FRAMES: usize = 14;
        for frame in 0..DESTABILIZE_FRAMES {
            let progress =
                frame as f64 / (DESTABILIZE_FRAMES - 1) as f64;
            let amplitude = if progress < 0.25 {
                0
            } else if progress < 0.65 {
                1
            } else {
                2
            };

            frames.push(Self::render(
                width,
                height,
                &glyphs,
                frame_number,
                |glyph| glyph.origin,
                0.0,
                amplitude,
            ));
            frame_number += 1;
        }

        const SCATTER_FRAMES: usize = 18;
        for frame in 0..SCATTER_FRAMES {
            let progress = (frame + 1) as f64 / SCATTER_FRAMES as f64;
            let eased = easing::out_expo(progress);

            frames.push(Self::render(
                width,
                height,
                &glyphs,
                frame_number,
                |glyph| glyph.origin.lerp(glyph.scattered, eased),
                0.0,
                1,
            ));
            frame_number += 1;
        }

        const UNSTABLE_HOLD_FRAMES: usize = 7;
        for _ in 0..UNSTABLE_HOLD_FRAMES {
            frames.push(Self::render(
                width,
                height,
                &glyphs,
                frame_number,
                |glyph| glyph.scattered,
                0.0,
                1,
            ));
            frame_number += 1;
        }

        const REASSEMBLE_FRAMES: usize = 24;
        for frame in 0..REASSEMBLE_FRAMES {
            let progress =
                (frame + 1) as f64 / REASSEMBLE_FRAMES as f64;
            let eased = easing::in_out_cubic(progress);
            let jitter_amplitude = if progress < 0.7 { 1 } else { 0 };

            frames.push(Self::render(
                width,
                height,
                &glyphs,
                frame_number,
                |glyph| glyph.scattered.lerp(glyph.origin, eased),
                easing::in_out_sine(progress),
                jitter_amplitude,
            ));
            frame_number += 1;
        }

        const SETTLE_FRAMES: usize = 6;
        for frame in 0..SETTLE_FRAMES {
            let jitter_amplitude =
                usize::from(frame < SETTLE_FRAMES / 2) as i32;

            frames.push(Self::render(
                width,
                height,
                &glyphs,
                frame_number,
                |glyph| glyph.origin,
                1.0,
                jitter_amplitude,
            ));
            frame_number += 1;
        }

        frames.push(Self::render(
            width,
            height,
            &glyphs,
            frame_number,
            |glyph| glyph.origin,
            1.0,
            0,
        ));

        frames
    }
}
