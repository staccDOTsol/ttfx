use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::easing::{Easing, QuadInOut};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};

use super::Effect;

pub struct Swarm;

impl Swarm {
    pub fn new() -> Self {
        Swarm
    }
}

impl Effect for Swarm {
    fn name(&self) -> &str {
        "swarm"
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
        let mut rng = Rng::new(0x2e4b5f6e);
        let mut ids: Vec<u32> = Vec::new();
        let mut starts: Vec<Coord> = Vec::new();
        let mut targets: Vec<Coord> = Vec::new();
        let mut progress: Vec<f64> = Vec::new();
        let mut next_id = 0u32;

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let start = Coord::new(
                    rng.range_i32(0, width as i32 - 1),
                    rng.range_i32(0, height as i32 - 1),
                );
                let target = Coord::new(col as i32, row as i32);
                let character = EffectCharacter::new(next_id, start, ch);
                terminal.add_character(character);

                ids.push(next_id);
                starts.push(start);
                targets.push(target);
                progress.push(rng.next_f64() * 0.35);
                next_id += 1;
            }
        }

        if ids.is_empty() {
            return vec![terminal.render_frame()];
        }

        let mut frames = Vec::new();
        let speed = 0.055;

        for _ in 0..80 {
            for i in 0..ids.len() {
                if progress[i] < 1.0 {
                    progress[i] = (progress[i] + speed).min(1.0);
                }

                let t = progress[i];
                let eased = QuadInOut.ease(t);
                let start = starts[i];
                let target = targets[i];

                let sway = ((t * std::f64::consts::PI * 2.0) + (i as f64 * 0.7)).sin() * 1.4;
                let x = start.x as f64 + (target.x - start.x) as f64 * eased + sway;
                let y = start.y as f64 + (target.y - start.y) as f64 * eased + sway * 0.5;
                let pos = Coord::new(x.round() as i32, y.round() as i32);

                if let Some(character) = terminal.get_character_mut(ids[i]) {
                    character.position = pos;
                    character.visible = true;
                    character.color_pair = ColorPair::new(Self::color_for(t, i), Color::BLACK);
                }
            }

            frames.push(terminal.render_frame());

            if progress.iter().all(|&p| p >= 1.0) {
                break;
            }
        }

        // Final frame: all characters at their input coordinates with white-on-black styling.
        for i in 0..ids.len() {
            if let Some(character) = terminal.get_character_mut(ids[i]) {
                character.position = targets[i];
                character.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
            }
        }
        frames.push(terminal.render_frame());

        frames
    }
}

impl Swarm {
    fn color_for(t: f64, idx: usize) -> Color {
        let palette = [
            Color::new(255, 60, 0),
            Color::new(255, 160, 0),
            Color::new(255, 255, 0),
            Color::new(0, 200, 80),
            Color::new(0, 170, 255),
            Color::new(180, 0, 255),
        ];

        let start = palette[idx % palette.len()];
        let end = Color::WHITE;

        let r = (start.r as f64 + (end.r as f64 - start.r as f64) * t).round() as u8;
        let g = (start.g as f64 + (end.g as f64 - start.g as f64) * t).round() as u8;
        let b = (start.b as f64 + (end.b as f64 - start.b as f64) * t).round() as u8;

        Color::new(r, g, b)
    }
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let r = self.next_f64();
        min + (r * (max - min + 1) as f64) as i32
    }
}
