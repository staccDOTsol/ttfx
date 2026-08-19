use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};

/// Simple deterministic pseudo-random generator (Linear Congruential Generator).
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_f64() * (max - min) as f64) as usize
    }
}

pub struct Thunderstorm {
    rain_color: Color,
    lightning_color: Color,
    frame_count: usize,
    width: u16,
    height: u16,
}

impl Thunderstorm {
    pub fn new() -> Self {
        Thunderstorm {
            rain_color: Color::new(0, 120, 255),     // blue
            lightning_color: Color::new(255, 255, 0), // yellow
            frame_count: 120,
            width: 80,
            height: 24,
        }
    }

    fn rain_frames(&self, input: &str, seed: u64) -> Vec<String> {
        let mut rng = Lcg::new(seed);
        let mut terminal = Terminal::new(self.width, self.height);

        // Place each input character at a random column near the top.
        for (i, ch) in input.chars().enumerate() {
            if ch == '\n' {
                continue;
            }
            let x = rng.next_range(0, self.width as usize) as i32;
            let y = rng.next_range(0, 3) as i32; // start near top
            let mut character = EffectCharacter::new(i as u32, Coord::new(x, y), ch);
            character.color_pair = ColorPair::new(self.rain_color, Color::BLACK);
            character.visible = true;
            terminal.add_character(character);
        }

        let mut frames = Vec::new();

        for frame_idx in 0..self.frame_count {
            // Lightning flash: every 20 frames, brighten all characters for one frame.
            let lightning = frame_idx % 20 == 0 && frame_idx > 0;

            for character in terminal.get_characters_mut() {
                character.color_pair = if lightning {
                    ColorPair::new(self.lightning_color, Color::WHITE)
                } else {
                    ColorPair::new(self.rain_color, Color::BLACK)
                };

                // Rain movement: move down one row each frame, wrapping to top.
                character.position.y = (character.position.y + 1) % self.height as i32;

                // Occasional horizontal drift.
                if rng.next_f64() < 0.1 {
                    character.position.x = rng.next_range(0, self.width as usize) as i32;
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }

    fn storm_frames(&self, input: &str) -> Vec<String> {
        // Fixed seed for reproducibility.
        self.rain_frames(input, 42)
    }
}

impl Effect for Thunderstorm {
    fn name(&self) -> &str {
        "thunderstorm"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        self.storm_frames(input)
    }
}
