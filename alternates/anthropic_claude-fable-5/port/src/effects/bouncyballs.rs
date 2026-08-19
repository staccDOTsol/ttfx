//! Bouncy balls effect: characters are replaced by colored "balls" that drop
//! from above the canvas and bounce (out_bounce easing) into their input
//! positions, column by column, then settle into a final gradient color.
//!
//! Port of terminaltexteffects/effects/effect_bouncyballs.py (simplified to
//! the engine facilities available in this crate).

use std::collections::{BTreeMap, HashSet};

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Deterministic little PRNG so the effect needs no external crates.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    /// Uniform-ish integer in `0..n` (returns 0 when `n == 0`).
    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            self.next_u32() as usize % n
        }
    }
}

pub struct Bouncyballs;

impl Bouncyballs {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Bouncyballs {
    fn name(&self) -> &str {
        "bouncyballs"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;
        let width = terminal.canvas.width;
        let mut rng = Rng::new(0x0b0a_11ce_b0b5);

        // Defaults from the Python effect config.
        let ball_colors: Vec<Color> = ["d1f4a5", "96e2a4", "5acda9"]
            .iter()
            .filter_map(|h| Color::from_hex(h))
            .collect();
        let ball_symbols = ['*', 'o', 'O', '0', '.'];
        let final_gradient = Gradient::new(
            &[
                Color::from_hex("f8ffae").expect("valid hex"),
                Color::from_hex("43c6ac").expect("valid hex"),
            ],
            12,
        );

        // Configure every character: random ball look, a starting position
        // above the canvas, and a bouncing path down to its input coordinate.
        let mut columns: BTreeMap<i32, Vec<u32>> = BTreeMap::new();
        for character in terminal.get_characters_mut() {
            let color = ball_colors[rng.below(ball_colors.len())];
            let symbol = ball_symbols[rng.below(ball_symbols.len())];
            let extra = rng.below(((height / 2).max(1)) as usize) as i32;
            character.motion.current_coord =
                Coord::new(character.input_coord.column, height + 1 + extra);
            character
                .animation
                .set_appearance(symbol, Some(ColorPair::fg_only(color)));
            let path = character
                .motion
                .new_path("input", 0.25, Some(easing::out_bounce as _));
            path.new_waypoint("input", character.input_coord);
            columns
                .entry(character.input_coord.column)
                .or_default()
                .push(character.character_id);
        }

        // Shuffle the column drop order (Fisher-Yates).
        let mut column_groups: Vec<Vec<u32>> = columns.into_values().collect();
        let mut i = column_groups.len();
        while i > 1 {
            i -= 1;
            let j = rng.below(i + 1);
            column_groups.swap(i, j);
        }
        column_groups.reverse(); // pop() from the back drops in shuffled order

        let mut falling: HashSet<u32> = HashSet::new();
        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let max_frames = 4000usize;
        let mut ticks_since_drop = 0u32;

        while (!column_groups.is_empty() || terminal.is_active() || !falling.is_empty())
            && frames.len() < max_frames
        {
            // Release the next column every couple of ticks.
            if !column_groups.is_empty() {
                ticks_since_drop += 1;
                if ticks_since_drop >= 2 {
                    ticks_since_drop = 0;
                    if let Some(group) = column_groups.pop() {
                        let ids: HashSet<u32> = group.iter().copied().collect();
                        for character in terminal.get_characters_mut() {
                            if ids.contains(&character.character_id) {
                                character.is_visible = true;
                                character.motion.activate_path("input");
                                falling.insert(character.character_id);
                            }
                        }
                    }
                }
            }

            terminal.tick();

            // Characters that just finished bouncing settle into the final
            // gradient color (diagonal direction, as in the Python default).
            let mut landed: Vec<u32> = Vec::new();
            for character in terminal.get_characters_mut() {
                if falling.contains(&character.character_id)
                    && character.motion.movement_is_complete()
                {
                    let denom = (width + height - 2).max(1) as f64;
                    let t = ((character.input_coord.column - 1)
                        + (character.input_coord.row - 1)) as f64
                        / denom;
                    let color = final_gradient
                        .get_color_at_fraction(t)
                        .unwrap_or(Color::new(255, 255, 255));
                    character
                        .animation
                        .set_appearance(character.input_symbol, Some(ColorPair::fg_only(color)));
                    landed.push(character.character_id);
                }
            }
            for id in landed {
                falling.remove(&id);
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
