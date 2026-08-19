//! Rain effect: characters fall from the top of the canvas like raindrops,
//! then settle into their input positions with a final gradient color.
//! Port of terminaltexteffects/effects/effect_rain.py.

use std::collections::HashSet;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Small deterministic xorshift64* PRNG (no external rng module in this crate).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform f64 in [0, 1).
    fn gen_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform usize in [lo, hi). Requires hi > lo.
    fn gen_range(&mut self, lo: usize, hi: usize) -> usize {
        lo + (self.next_u64() % (hi - lo) as u64) as usize
    }

    fn choice<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.gen_range(0, items.len())]
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        // Fisher-Yates
        for i in (1..items.len()).rev() {
            let j = self.gen_range(0, i + 1);
            items.swap(i, j);
        }
    }
}

/// The rain effect. Mirrors the Python `Rain` effect and its default config:
/// rain drop symbols, a palette of blue rain colors, random per-drop movement
/// speed, and a final vertical gradient applied once drops reach home.
pub struct Rain {
    rain_symbols: Vec<char>,
    rain_colors: Vec<Color>,
    movement_speed_range: (f64, f64),
    final_gradient_stops: Vec<Color>,
    final_gradient_steps: usize,
    max_frames: usize,
}

impl Rain {
    pub fn new() -> Self {
        // Defaults from the Python RainConfig.
        let rain_colors = [
            "00315C", "004C8F", "0075DB", "3F91D9", "78B9F2", "9AC8F5", "B8D8F8", "E3EFFC",
        ]
        .iter()
        .filter_map(|h| Color::from_hex(h))
        .collect();
        let final_gradient_stops = ["488bff", "b2e7de", "57eaf7"]
            .iter()
            .filter_map(|h| Color::from_hex(h))
            .collect();
        Self {
            rain_symbols: vec!['o', '.', ',', '*', '|'],
            rain_colors,
            movement_speed_range: (0.1, 0.2),
            final_gradient_stops,
            final_gradient_steps: 12,
            max_frames: 2000,
        }
    }
}

impl Default for Rain {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Rain {
    fn name(&self) -> &str {
        "rain"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;
        let mut rng = Rng::new(0xA1B2_C3D4_E5F6_0718 ^ input.len() as u64);

        let final_gradient = Gradient::new(&self.final_gradient_stops, self.final_gradient_steps);

        // Per-character setup: place at the top of the character's column,
        // style as a raindrop, and build a path home to the input coordinate.
        // Remember each character's final (settled) visual.
        let mut final_visuals: Vec<(u32, char, Option<ColorPair>)> = Vec::new();
        let mut pending: Vec<u32> = Vec::new();

        for character in terminal.get_characters_mut() {
            let drop_symbol = *rng.choice(&self.rain_symbols);
            let drop_color = *rng.choice(&self.rain_colors);
            let (lo, hi) = self.movement_speed_range;
            let speed = lo + rng.gen_f64() * (hi - lo);

            // Start at the top of the canvas in the character's column.
            character.motion.current_coord = Coord::new(character.input_coord.column, height);

            // Path from the top down to the input coordinate.
            let path = character
                .motion
                .new_path("input", speed, Some(easing::in_quad as fn(f64) -> f64));
            path.new_waypoint("input", character.input_coord);

            // Raindrop appearance while falling.
            character
                .animation
                .set_appearance(drop_symbol, Some(ColorPair::fg_only(drop_color)));

            // Final gradient color based on the character's row (vertical direction).
            let fraction = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient.get_color_at_fraction(fraction);
            let final_pair = final_color.map(ColorPair::fg_only);
            final_visuals.push((character.character_id, character.input_symbol, final_pair));

            pending.push(character.character_id);
        }

        // Release order is random, as in the Python effect.
        rng.shuffle(&mut pending);
        // Pop from the back cheaply; order is already random.
        pending.reverse();

        let mut settled: HashSet<u32> = HashSet::new();
        let mut released: HashSet<u32> = HashSet::new();

        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        while (!pending.is_empty() || terminal.is_active()) && frames.len() < self.max_frames {
            // Release 1-3 raindrops per tick.
            if !pending.is_empty() {
                let count = rng.gen_range(1, 4).min(pending.len());
                let mut to_release: HashSet<u32> = HashSet::new();
                for _ in 0..count {
                    if let Some(id) = pending.pop() {
                        to_release.insert(id);
                    }
                }
                for character in terminal.get_characters_mut() {
                    if to_release.contains(&character.character_id) {
                        character.is_visible = true;
                        character.motion.activate_path("input");
                        released.insert(character.character_id);
                    }
                }
            }

            terminal.tick();

            // When a drop reaches home, settle it into the input symbol with
            // its final gradient color.
            for character in terminal.get_characters_mut() {
                if released.contains(&character.character_id)
                    && !settled.contains(&character.character_id)
                    && character.motion.movement_is_complete()
                {
                    if let Some((_, symbol, colors)) = final_visuals
                        .iter()
                        .find(|(id, _, _)| *id == character.character_id)
                    {
                        character.animation.set_appearance(*symbol, *colors);
                    }
                    settled.insert(character.character_id);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
