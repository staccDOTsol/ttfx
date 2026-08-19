//! Wipe effect: performs a directional wipe across the terminal, revealing
//! characters column by column while fading them through a gradient toward
//! their final color. Port of terminaltexteffects/effects/effect_wipe.py.

use std::collections::BTreeMap;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Number of interpolation steps between gradient stops (Python:
/// `final_gradient_steps`).
const FINAL_GRADIENT_STEPS: usize = 12;

/// Ticks each gradient frame is shown (Python: `final_gradient_frames`).
const FINAL_GRADIENT_FRAMES: u32 = 5;

/// Ticks to wait between wiping each group (Python: `wipe_delay`, default 0).
const WIPE_DELAY: u32 = 0;

/// Safety cap on the number of rendered frames.
const MAX_FRAMES: usize = 4000;

/// The wipe effect. Wipes the text from left to right (the Python default
/// `wipe_direction = "column_left_to_right"`), styling each character with a
/// gradient scene that settles on its final gradient color.
pub struct Wipe;

impl Wipe {
    pub fn new() -> Self {
        Wipe
    }

    /// Default gradient stops shared with the Python effect defaults.
    fn gradient_stops() -> [Color; 3] {
        [
            Color::from_hex("833ab4").expect("valid hex"),
            Color::from_hex("fd1d1d").expect("valid hex"),
            Color::from_hex("fcb045").expect("valid hex"),
        ]
    }
}

impl Default for Wipe {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Wipe {
    fn name(&self) -> &str {
        "wipe"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());

        if terminal.get_characters().is_empty() {
            return vec![terminal.render_frame()];
        }

        let final_gradient = Gradient::new(&Self::gradient_stops(), FINAL_GRADIENT_STEPS);
        let base_color = *final_gradient
            .spectrum
            .first()
            .expect("gradient spectrum is non-empty");
        let height = terminal.canvas.height;

        // --- build(): per-character final color and wipe gradient scene -----
        for character in terminal.get_characters_mut() {
            // Vertical coordinate-based color mapping, mirroring
            // Gradient.build_coordinate_color_mapping(..., Direction.VERTICAL).
            let t = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                1.0
            };
            let final_color = final_gradient.get_color_at_fraction(t).unwrap_or(base_color);

            // Gradient from the first spectrum color to this character's
            // final color; one frame per color (apply_gradient_to_symbols).
            let wipe_gradient = Gradient::new(&[base_color, final_color], FINAL_GRADIENT_STEPS);
            let symbol = character.input_symbol;
            let scene = character.animation.new_scene("wipe", false);
            for color in &wipe_gradient.spectrum {
                scene.add_frame(symbol, FINAL_GRADIENT_FRAMES, Some(ColorPair::fg_only(*color)));
            }
        }

        // --- grouping: column_left_to_right ---------------------------------
        let mut columns: BTreeMap<i32, Vec<u32>> = BTreeMap::new();
        for character in terminal.get_characters() {
            columns
                .entry(character.input_coord.column)
                .or_default()
                .push(character.character_id);
        }
        // Within a column, order top-to-bottom (row descending) like the
        // Python COLUMN_LEFT_TO_RIGHT grouping. Ids were allocated in
        // top-to-bottom reading order, so allocation order already matches;
        // sort explicitly to be safe.
        let groups: Vec<Vec<u32>> = columns.into_values().collect();

        // --- frame loop ------------------------------------------------------
        let mut frames: Vec<String> = Vec::new();
        let mut group_index: usize = 0;
        let mut wipe_delay_remaining: u32 = 0;

        loop {
            if group_index < groups.len() {
                if wipe_delay_remaining == 0 {
                    // Reveal the next group and start its wipe scenes.
                    let group = &groups[group_index];
                    for id in group {
                        terminal.set_character_visibility(*id, true);
                    }
                    for character in terminal.get_characters_mut() {
                        if group.contains(&character.character_id) {
                            character.animation.activate_scene("wipe");
                        }
                    }
                    group_index += 1;
                    wipe_delay_remaining = WIPE_DELAY;
                } else {
                    wipe_delay_remaining -= 1;
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());

            let done = group_index >= groups.len() && !terminal.is_active();
            if done || frames.len() >= MAX_FRAMES {
                break;
            }
        }

        frames
    }
}
