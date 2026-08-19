use super::Effect;
use crate::engine::{EffectCharacter, Terminal};
use crate::utils::easing::{get_easing, Easing};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Spray;

impl Spray {
    pub fn new() -> Self {
        Spray
    }
}

impl Effect for Spray {
    fn name(&self) -> &str {
        "spray"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let chars: Vec<char> = input.chars().collect();
        if chars.is_empty() {
            return Vec::new();
        }

        // Build a terminal canvas that is wide enough to display all characters,
        // wrapping long input onto multiple rows.
        let width: u16 = 80;
        let needed_height = ((chars.len() + width as usize - 1) / width as usize) as u16;
        let height = needed_height.max(10);
        let mut terminal = Terminal::new(width, height);

        // A simple color gradient from red through green to blue.
        let gradient = Gradient::new(vec![
            (0.0, Color::RED),
            (0.5, Color::GREEN),
            (1.0, Color::BLUE),
        ]);

        // Deterministic pseudo-random positions based on the input characters.
        let seed = chars.iter().map(|c| *c as u64).sum::<u64>().max(1);
        let mut rng = XorShift::new(seed);
        let mut initial_positions = Vec::with_capacity(chars.len());

        for (i, &ch) in chars.iter().enumerate() {
            let start_x = (rng.next_u64() % width as u64) as i32;
            let start_y = (rng.next_u64() % height as u64) as i32;
            initial_positions.push(Coord::new(start_x, start_y));

            let t = i as f64 / chars.len().max(1) as f64;
            let fg = gradient.color_at(t);
            let color_pair = ColorPair::new(fg, Color::BLACK);

            let mut character = EffectCharacter::new(i as u32, Coord::new(start_x, start_y), ch);
            character.color_pair = color_pair;
            character.visible = true;
            character.bold = i % 3 == 0;
            terminal.add_character(character);
        }

        let total_frames: u32 = 20;
        let mut frames = Vec::with_capacity(total_frames as usize + 1);
        let ease = get_easing("sine_out").expect("sine_out easing should exist");

        for frame in 0..=total_frames {
            let t = frame as f64 / total_frames as f64;
            let eased = ease.ease(t);

            {
                let characters = terminal.get_characters_mut();
                for (i, character) in characters.iter_mut().enumerate() {
                    let target = Coord::new((i as u16 % width) as i32, (i as u16 / width) as i32);
                    let start = initial_positions[i];
                    let x = start.x as f64 + (target.x - start.x) as f64 * eased;
                    let y = start.y as f64 + (target.y - start.y) as f64 * eased;
                    character.position = Coord::new(x.round() as i32, y.round() as i32);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}

/// A tiny xorshift64* PRNG, used only to avoid external dependencies.
struct XorShift {
    state: u64,
}

impl XorShift {
    fn new(seed: u64) -> Self {
        XorShift { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}
