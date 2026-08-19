//! random_sequence: characters are revealed in random order, each fading in
//! from a starting color to its final gradient color (port of
//! terminaltexteffects/effects/effect_random_sequence.py).

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Tiny deterministic PRNG (xorshift64*) used for shuffling the reveal order,
/// mirroring the Python effect's `random.shuffle` of pending characters.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform-ish integer in `0..bound` (bound must be > 0).
    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}

/// Python config defaults for the random_sequence effect.
const STARTING_COLOR: (u8, u8, u8) = (0x00, 0x00, 0x00);
const FINAL_GRADIENT_STOPS: [&str; 3] = ["8A008A", "00D1FF", "FFFFFF"];
const FINAL_GRADIENT_STEPS: usize = 12;
const FINAL_GRADIENT_FRAMES: u32 = 12;
const FADE_STEPS: usize = 7;
const SPEED: f64 = 0.004;
const MAX_FRAMES: usize = 20_000;

pub struct RandomSequence;

impl RandomSequence {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RandomSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for RandomSequence {
    fn name(&self) -> &str {
        "random_sequence"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());

        // Build the final gradient (vertical direction, as upstream default).
        let stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .filter_map(|hex| Color::from_hex(hex))
            .collect();
        let final_gradient = Gradient::new(&stops, FINAL_GRADIENT_STEPS);
        let starting_color = Color::new(STARTING_COLOR.0, STARTING_COLOR.1, STARTING_COLOR.2);

        let canvas_height = terminal.canvas.height.max(1);

        // Per-character build phase: hide the character and prepare its
        // fade-in scene from the starting color to its final gradient color.
        let mut pending: Vec<u32> = Vec::new();
        for character in terminal.get_characters_mut() {
            character.is_visible = false;

            // Vertical gradient mapping: fraction of the character's row
            // through the canvas height selects its final color.
            let fraction = if canvas_height > 1 {
                (character.input_coord.row - 1) as f64 / (canvas_height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(Color::new(0xFF, 0xFF, 0xFF));

            let fade_gradient = Gradient::new(&[starting_color, final_color], FADE_STEPS);
            let scene = character.animation.new_scene("fade_in", false);
            for color in &fade_gradient.spectrum {
                scene.add_frame(
                    character.input_symbol,
                    FINAL_GRADIENT_FRAMES,
                    Some(ColorPair::fg_only(*color)),
                );
            }

            pending.push(character.character_id);
        }

        // Shuffle the reveal order (Python: random.shuffle(self.pending_chars)).
        let mut rng = Rng::new(0xC0FFEE ^ pending.len() as u64);
        rng.shuffle(&mut pending);

        // Number of characters activated per tick:
        // max(int(speed * total), 1) as in the Python effect.
        let total = pending.len();
        let chars_per_tick = ((SPEED * total as f64) as usize).max(1);

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let mut frame_count = 0usize;
        while (!pending.is_empty() || terminal.is_active()) && frame_count < MAX_FRAMES {
            // Activate the next batch of characters.
            for _ in 0..chars_per_tick {
                if let Some(id) = pending.pop() {
                    if let Some(character) = terminal
                        .get_characters_mut()
                        .iter_mut()
                        .find(|c| c.character_id == id)
                    {
                        character.is_visible = true;
                        character.animation.activate_scene("fade_in");
                    }
                } else {
                    break;
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());
            frame_count += 1;
        }

        frames
    }
}
