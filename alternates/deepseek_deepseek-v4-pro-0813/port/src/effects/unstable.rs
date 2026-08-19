use std::f64::consts::PI;

use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::easing::{Easing, SineInOut};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Unstable;

impl Unstable {
    pub fn new() -> Self {
        Unstable
    }
}

impl Effect for Unstable {
    fn name(&self) -> &str {
        "unstable"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1) as u16;
        let height = lines.len().max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut origins = Vec::new();
        let mut next_id = 0u32;

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let mut character = EffectCharacter::new(next_id, coord, ch);
                character.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
                terminal.add_character(character);
                origins.push(coord);
                next_id += 1;
            }
        }

        let total_frames: usize = 60;
        let mut frames = Vec::with_capacity(total_frames);

        let gradient = Gradient::new(vec![
            (0.0, Color::new(255, 255, 255)),
            (0.3, Color::new(255, 140, 0)),
            (0.65, Color::new(220, 20, 60)),
            (1.0, Color::new(160, 32, 240)),
        ]);

        for frame_index in 0..total_frames {
            let t = frame_index as f64 / total_frames as f64;
            let phase = if t < 0.5 { t / 0.5 } else { (1.0 - t) / 0.5 };
            let eased = SineInOut.ease(phase);

            let mut updates = Vec::new();

            for (i, _character) in terminal.get_characters().iter().enumerate() {
                let origin = origins[i];
                let seed = ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                    ^ 0xBF58_476D_1CE4_E5B9;
                let angle = ((seed % 360) as f64) * PI / 180.0;
                let max_dist = 3.0 + ((seed >> 8) % 8) as f64;
                let dist = eased * max_dist;

                let jitter_base = ((seed % 360) as f64) * PI / 180.0;
                let jitter_amount = eased * (1.0 - eased) * 4.0;
                let jitter_x = ((frame_index as f64 * 0.9) + jitter_base).sin() * jitter_amount;
                let jitter_y = ((frame_index as f64 * 0.7) + jitter_base * 0.8).cos() * jitter_amount;

                let new_x = (origin.x as f64 + dist * angle.cos() + jitter_x).round() as i32;
                let new_y = (origin.y as f64 + dist * angle.sin() + jitter_y).round() as i32;

                let color_t = (t + ((i % 13) as f64 * 0.07)) % 1.0;
                let color = gradient.color_at(color_t);

                updates.push((i as u32, Coord::new(new_x, new_y), color));
            }

            for (character_id, coord, color) in updates {
                if let Some(character) = terminal.get_character_mut(character_id) {
                    character.position = coord;
                    character.color_pair = ColorPair::new(color, Color::BLACK);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
