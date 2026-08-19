use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn range(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        (self.next() % ((max - min + 1) as u64)) as i32 + min
    }
}

pub struct Crumble;

impl Crumble {
    pub fn new() -> Self {
        Crumble
    }
}

fn dim_color(color: Color, factor: f64) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    Color::new(
        (color.r as f64 * factor).round() as u8,
        (color.g as f64 * factor).round() as u8,
        (color.b as f64 * factor).round() as u8,
    )
}

impl Effect for Crumble {
    fn name(&self) -> &str {
        "crumble"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut originals: Vec<Color> = Vec::new();
        let mut delays: Vec<i32> = Vec::new();
        let mut start_y: Vec<i32> = Vec::new();
        let mut rng = Rng::new(0x5eed_cafe);

        let base_gradient = Gradient::new(vec![
            (0.0, Color::new(255, 180, 0)),
            (0.5, Color::new(255, 90, 0)),
            (1.0, Color::new(180, 0, 20)),
        ]);

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let coord = Coord::new(x as i32, y as i32);
                let id = originals.len() as u32;
                let mut character = EffectCharacter::new(id, coord, ch);

                let t = if height > 1 {
                    y as f64 / (height - 1) as f64
                } else {
                    0.5
                };
                let color = base_gradient.color_at(t);
                character.color_pair = ColorPair::new(color, Color::BLACK);

                originals.push(color);
                delays.push(rng.range(0, 12));
                start_y.push(y as i32);
                terminal.add_character(character);
            }
        }

        let mut frames: Vec<String> = Vec::with_capacity(64);
        frames.push(terminal.render_frame());

        if terminal.get_characters().is_empty() {
            return frames;
        }

        let total_frames = 48;

        for frame_idx in 0..total_frames {
            {
                let chars = terminal.get_characters_mut();
                for (i, character) in chars.iter_mut().enumerate() {
                    if !character.visible {
                        continue;
                    }

                    let delay = delays[i];

                    if frame_idx < delay {
                        // Slightly dim while waiting to crumble.
                        let fade =
                            1.0 - (frame_idx as f64 / (delay as f64 + 1.0)) * 0.15;
                        character.color_pair.fg = dim_color(originals[i], fade);
                        continue;
                    }

                    let speed = 1 + ((i as i32 + frame_idx as i32) % 2);
                    character.position.y += speed;

                    let fall_distance = character.position.y - start_y[i];
                    let max_fall = (height as i32 - start_y[i]).max(1) + 2;
                    let progress =
                        (fall_distance as f64 / max_fall as f64).clamp(0.0, 1.0);
                    let fade = (1.0 - progress).max(0.15);
                    character.color_pair.fg = dim_color(originals[i], fade);

                    if character.position.y >= height as i32 {
                        character.visible = false;
                    }
                }
            }
            frames.push(terminal.render_frame());
        }

        frames
    }
}
