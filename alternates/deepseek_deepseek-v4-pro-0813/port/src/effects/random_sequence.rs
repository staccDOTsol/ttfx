use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct RandomSequence;

impl RandomSequence {
    pub fn new() -> Self {
        RandomSequence
    }
}

impl Effect for RandomSequence {
    fn name(&self) -> &str {
        "random_sequence"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Vec::new();
        }

        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);

        // Create one character per visible input character, hidden initially.
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let id = terminal.characters.len() as u32;
                let coord = Coord::new(col as i32, row as i32);
                let mut character = EffectCharacter::new(id, coord, ch);
                character.visible = false;
                terminal.add_character(character);
            }
        }

        if terminal.get_characters().is_empty() {
            return Vec::new();
        }

        // Deterministic PRNG so repeated runs with the same input are stable.
        let mut seed = hash_u64(input);
        if seed == 0 {
            seed = 0x9e3779b97f4a7c15;
        }
        let mut order: Vec<u32> = terminal.get_characters().iter().map(|c| c.id).collect();
        shuffle(&mut order, &mut seed);

        let gradient = Gradient::new(vec![
            (0.0, Color::new(255, 0, 0)),
            (0.25, Color::new(255, 165, 0)),
            (0.5, Color::new(0, 255, 0)),
            (0.75, Color::new(0, 255, 255)),
            (1.0, Color::new(255, 0, 255)),
        ]);

        let total = order.len().max(1);
        let mut frames = Vec::with_capacity(order.len() + 2);

        // Initial all-hidden frame still carries SGR styling from the canvas renderer.
        frames.push(terminal.render_frame());

        for (index, id) in order.iter().enumerate() {
            {
                if let Some(character) = terminal.get_character_mut(*id) {
                    let t = index as f64 / (total - 1) as f64;
                    character.color_pair = ColorPair::new(gradient.color_at(t), Color::BLACK);
                    character.visible = true;
                }
            }
            frames.push(terminal.render_frame());
        }

        // Extra hold frame so the final state is visible for a moment.
        frames.push(terminal.render_frame());
        frames
    }
}

fn hash_u64(s: &str) -> u64 {
    let mut state = 0xcbf29ce484222325u64;
    for byte in s.bytes() {
        state ^= byte as u64;
        state = state.wrapping_mul(0x100000001b3);
    }
    state
}

fn rand_u64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn shuffle<T>(slice: &mut [T], state: &mut u64) {
    for i in (1..slice.len()).rev() {
        let j = (rand_u64(state) as usize) % (i + 1);
        slice.swap(i, j);
    }
}
