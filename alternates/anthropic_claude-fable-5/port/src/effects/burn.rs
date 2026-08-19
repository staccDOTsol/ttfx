//! Burn effect (port of terminaltexteffects/effects/effect_burn.py).
//!
//! Characters start visible in a dim "unburned" color, then ignite one after
//! another from the bottom of each column upward, left to right. Each burning
//! character cycles through a vertical build order of block symbols colored by
//! a fire gradient, then cools from the last fire color into its final
//! gradient color.

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Color of the text before it ignites (Python: `starting_color`).
const STARTING_COLOR: &str = "837373";

/// Fire gradient stops, white-hot through ember (Python: `burn_colors`).
const BURN_COLORS: [&str; 5] = ["ffffff", "fff75d", "fe650d", "8A003C", "510100"];

/// Final gradient stops applied vertically (Python: `final_gradient_stops`).
const FINAL_GRADIENT_STOPS: [&str; 2] = ["00c3ff", "ffff1c"];

/// Symbols the character passes through while burning
/// (Python: `vertical_build_order`).
const VERTICAL_BUILD_ORDER: [char; 9] = ['\'', '.', '▖', '▙', '█', '▜', '▀', '▝', '.'];

/// Ticks each burn frame is held.
const FRAME_DURATION: u32 = 2;

/// Small deterministic PRNG standing in for Python's `random.randint`,
/// used to stagger how many characters ignite per tick.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    /// Uniform value in `lo..=hi`.
    fn range_inclusive(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        lo + self.next_u32() % (hi - lo + 1)
    }
}

/// The burn effect.
pub struct Burn;

impl Burn {
    pub fn new() -> Self {
        Burn
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
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;

        let starting_color =
            Color::from_hex(STARTING_COLOR).expect("valid starting color hex");
        let burn_stops: Vec<Color> = BURN_COLORS
            .iter()
            .map(|hex| Color::from_hex(hex).expect("valid burn color hex"))
            .collect();
        let final_stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .map(|hex| Color::from_hex(hex).expect("valid final gradient hex"))
            .collect();

        let fire_gradient = Gradient::new(&burn_stops, 12);
        let final_gradient = Gradient::new(&final_stops, 12);
        let last_fire_color = *fire_gradient
            .spectrum
            .last()
            .expect("fire gradient has stops");

        // Build the burn scene for every character. The Python effect chains a
        // "burn" scene into a "burned" cool-down scene via an event handler;
        // here the cool-down frames are appended to the same scene so the
        // whole sequence plays through in one activation.
        for character in terminal.get_characters_mut() {
            // Final color mapped by vertical position (bottom row -> first stop).
            let row_fraction = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(row_fraction)
                .unwrap_or(starting_color);

            // Everything is visible in the unburned color from the start.
            character.is_visible = true;
            character
                .animation
                .set_appearance(character.input_symbol, Some(ColorPair::fg_only(starting_color)));

            let input_symbol = character.input_symbol;
            let scene = character.animation.new_scene("burn", false);

            // Fire phase: distribute the fire gradient across the vertical
            // build order symbols (mirrors apply_gradient_to_symbols).
            let n_colors = fire_gradient.spectrum.len();
            for (i, color) in fire_gradient.spectrum.iter().enumerate() {
                let symbol_index =
                    (i * VERTICAL_BUILD_ORDER.len() / n_colors).min(VERTICAL_BUILD_ORDER.len() - 1);
                scene.add_frame(
                    VERTICAL_BUILD_ORDER[symbol_index],
                    FRAME_DURATION,
                    Some(ColorPair::fg_only(*color)),
                );
            }

            // Cool-down phase: fade from the last ember color to the
            // character's final gradient color on its own symbol.
            let cooled = Gradient::new(&[last_fire_color, final_color], 12);
            for color in &cooled.spectrum {
                scene.add_frame(input_symbol, FRAME_DURATION, Some(ColorPair::fg_only(*color)));
            }
        }

        // Ignition order: column left-to-right, row bottom-to-top.
        let mut order: Vec<(i32, i32, u32)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.input_coord.column, c.input_coord.row, c.character_id))
            .collect();
        order.sort();
        // Reverse so `pop()` yields the front of the ignition order.
        let mut pending: Vec<u32> = order.into_iter().rev().map(|(_, _, id)| id).collect();

        let mut rng = Lcg::new(0x5eed_0bad_c0ff_ee11);
        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        let max_frames = 10_000usize;
        let mut count = 0usize;
        while (!pending.is_empty() || terminal.is_active()) && count < max_frames {
            // Ignite a few characters per tick (Python: random.randint(2, 4)).
            let ignite_count = rng.range_inclusive(2, 4);
            for _ in 0..ignite_count {
                match pending.pop() {
                    Some(id) => {
                        if let Some(character) = terminal
                            .get_characters_mut()
                            .iter_mut()
                            .find(|c| c.character_id == id)
                        {
                            character.is_visible = true;
                            character.animation.activate_scene("burn");
                        }
                    }
                    None => break,
                }
            }
            terminal.tick();
            frames.push(terminal.render_frame());
            count += 1;
        }

        frames
    }
}
