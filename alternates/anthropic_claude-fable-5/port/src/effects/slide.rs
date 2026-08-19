//! Slide effect: characters slide into position from outside the canvas,
//! group (row) by group, while their color eases from the first gradient
//! stop toward their final gradient color.
//!
//! Port of terminaltexteffects/effects/effect_slide.py with the upstream
//! defaults: row grouping, no merge, no reverse, movement_speed 0.5,
//! in_out_quad easing, gap 3, gradient stops 833ab4 -> fd1d1d -> fcb045.

use std::collections::{BTreeMap, VecDeque};

use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const MOVEMENT_SPEED: f64 = 0.5;
const GAP: u32 = 3;
const FINAL_GRADIENT_STEPS: usize = 12;
const FINAL_GRADIENT_FRAMES: u32 = 10;
const CHARACTER_GRADIENT_STEPS: usize = 10;
const MAX_FRAMES: usize = 5000;

/// Slide effect. See the Python original `effect_slide.py`.
pub struct Slide;

impl Slide {
    pub fn new() -> Self {
        Slide
    }
}

impl Effect for Slide {
    fn name(&self) -> &str {
        "slide"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;

        // Final gradient (vertical direction: color chosen by row).
        let stops = [
            Color::from_hex("833ab4").expect("valid hex"),
            Color::from_hex("fd1d1d").expect("valid hex"),
            Color::from_hex("fcb045").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&stops, FINAL_GRADIENT_STEPS);
        let start_color = final_gradient
            .spectrum
            .first()
            .copied()
            .unwrap_or(Color::new(255, 255, 255));

        // Group characters by row, top-to-bottom; left-to-right within a row.
        let mut rows: BTreeMap<i32, Vec<(i32, CharacterId)>> = BTreeMap::new();
        for character in terminal.get_characters() {
            rows.entry(character.input_coord.row)
                .or_default()
                .push((character.input_coord.column, character.character_id));
        }
        let mut groups: Vec<Vec<CharacterId>> = Vec::new();
        for (_row, mut members) in rows.into_iter().rev() {
            members.sort_by_key(|(column, _)| *column);
            groups.push(members.into_iter().map(|(_, id)| id).collect());
        }

        // Per-character setup: off-canvas starting coordinate (left of the
        // canvas, as with merge=False/reverse=False upstream), the
        // "input_path" back home, and the color gradient scene.
        for character in terminal.get_characters_mut() {
            let input_coord = character.input_coord;
            let symbol = character.input_symbol;
            let row = input_coord.row;

            let fraction = if height > 1 {
                (row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(start_color);

            // Start just outside the left edge on the character's row.
            character.motion.current_coord = Coord::new(0, row);

            let path = character
                .motion
                .new_path("input_path", MOVEMENT_SPEED, Some(easing::in_out_quad));
            path.new_waypoint("input_coord", input_coord);

            let char_gradient =
                Gradient::new(&[start_color, final_color], CHARACTER_GRADIENT_STEPS);
            let scene = character.animation.new_scene("gradient", false);
            for color in &char_gradient.spectrum {
                scene.add_frame(symbol, FINAL_GRADIENT_FRAMES, Some(ColorPair::fg_only(*color)));
            }

            // Carry color from the very first rendered cell onward.
            character
                .animation
                .set_appearance(symbol, Some(ColorPair::fg_only(start_color)));
        }

        // Frame loop: launch one group every GAP frames, then tick to
        // completion.
        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let mut pending_groups: VecDeque<Vec<CharacterId>> = groups.into();
        let mut current_gap = GAP; // launch the first group immediately

        while (!pending_groups.is_empty() || terminal.is_active()) && frames.len() < MAX_FRAMES {
            if !pending_groups.is_empty() {
                if current_gap >= GAP {
                    let group = pending_groups.pop_front().expect("non-empty deque");
                    for id in group {
                        if let Some(character) = terminal
                            .get_characters_mut()
                            .iter_mut()
                            .find(|c| c.character_id == id)
                        {
                            character.is_visible = true;
                            character.animation.activate_scene("gradient");
                            character.motion.activate_path("input_path");
                        }
                    }
                    current_gap = 0;
                } else {
                    current_gap += 1;
                }
            }
            terminal.tick();
            frames.push(terminal.render_frame());
        }

        frames
    }
}
