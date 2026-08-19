use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use std::time::{SystemTime, UNIX_EPOCH};

fn random_unit(seed: u64, idx: u64, frame: u64) -> f64 {
    let mut x = seed
        ^ idx.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ frame.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 30;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 27;
    x = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
    x ^= x >> 31;
    (x as f64 / u64::MAX as f64) - 0.5
}

pub struct Burn {
    fire_gradient: Gradient,
}

impl Burn {
    pub fn new() -> Self {
        let fire_gradient = Gradient::new(vec![
            (0.0, Color::new(255, 224, 138)),
            (0.15, Color::new(245, 185, 66)),
            (0.35, Color::new(245, 143, 60)),
            (0.55, Color::new(239, 93, 59)),
            (0.75, Color::new(216, 35, 38)),
            (0.9, Color::new(166, 7, 54)),
            (1.0, Color::new(70, 5, 18)),
        ]);
        Self { fire_gradient }
    }

    fn seed(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}

impl Default for Burn {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Burn {
    fn name(&self) -> &str {
        "burn"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = if input.trim().is_empty() {
            vec![" "]
        } else {
            input.lines().collect()
        };

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1) as u16;
        let height = lines.len().max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut original_symbols: Vec<char> = Vec::new();
        let mut original_coords: Vec<Coord> = Vec::new();

        for (line_index, line) in lines.iter().enumerate() {
            for (col_index, ch) in line.chars().enumerate() {
                let coord = Coord::new(col_index as i32, line_index as i32);
                let id = (line_index * width as usize + col_index) as u32;
                let mut character = EffectCharacter::new(id, coord, ch);
                character.color_pair = ColorPair::new(Color::new(255, 224, 138), Color::BLACK);
                original_symbols.push(ch);
                original_coords.push(coord);
                terminal.add_character(character);
            }
        }

        let seed = self.seed();
        let total_frames = 45usize.max(height as usize * 6) + 10;
        let burn_duration: f64 = 14.0;
        let mut frames = Vec::with_capacity(total_frames);

        for frame_idx in 0..total_frames {
            {
                let characters = terminal.get_characters_mut();
                for (idx, character) in characters.iter_mut().enumerate() {
                    let original = original_coords[idx];
                    let jitter = random_unit(seed, idx as u64, frame_idx as u64) * 2.5;
                    let start_frame = (height as i32 - 1 - original.y) as f64
                        / height as f64
                        * (total_frames as f64 * 0.55)
                        + jitter;
                    let p = (frame_idx as f64 - start_frame) / burn_duration;

                    if p < 0.0 {
                        character.color_pair =
                            ColorPair::new(Color::new(255, 224, 138), Color::BLACK);
                        character.symbol = original_symbols[idx];
                        character.bold = false;
                        character.dim = false;
                    } else if p < 1.0 {
                        character.color_pair = ColorPair::new(
                            self.fire_gradient.color_at(p.clamp(0.0, 1.0)),
                            Color::BLACK,
                        );
                        character.symbol = original_symbols[idx];
                        character.bold = p > 0.4;
                        character.dim = false;
                    } else {
                        character.color_pair =
                            ColorPair::new(self.fire_gradient.color_at(1.0), Color::BLACK);
                        character.symbol = if p < 1.25 { '*' } else { ' ' };
                        character.bold = false;
                        character.dim = true;
                    }
                }
            }
            frames.push(terminal.render_frame());
        }

        frames
    }
}
