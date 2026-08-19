//! Slice effect: the text is sliced into halves that slide into place from
//! opposite edges of the canvas.
//!
//! Port of `terminaltexteffects/effects/effect_slice.py` using the default
//! configuration: `slice_direction = "vertical"` (each row is cut at the text
//! center column; the left half drops in from above the canvas while the
//! right half rises in from below), `movement_speed = 0.15`, and the default
//! final gradient (`8A008A -> 00D1FF -> FFFFFF`) applied vertically across
//! the text. The upstream default easing (`in_out_expo`) is not available in
//! this engine's easing subset, so the closest available curve
//! (`in_out_cubic`) is used.

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Default movement speed (upstream `--movement-speed`).
const MOVEMENT_SPEED: f64 = 0.15;

/// Safety bound on the number of simulation ticks.
const MAX_FRAMES: usize = 2000;

pub struct Slice;

impl Slice {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Slice {
    fn name(&self) -> &str {
        "slice"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());

        if terminal.get_characters().is_empty() {
            return vec![terminal.render_frame()];
        }

        // Final gradient stops (Python defaults), 12 steps between stops.
        let stops = [
            Color::from_hex("8A008A").expect("valid hex literal"),
            Color::from_hex("00D1FF").expect("valid hex literal"),
            Color::from_hex("FFFFFF").expect("valid hex literal"),
        ];
        let final_gradient = Gradient::new(&stops, 12);

        // Text bounds, used for the gradient mapping (vertical direction) and
        // for the text center column where each row is sliced.
        let mut min_row = i32::MAX;
        let mut max_row = i32::MIN;
        let mut min_col = i32::MAX;
        let mut max_col = i32::MIN;
        for character in terminal.get_characters() {
            min_row = min_row.min(character.input_coord.row);
            max_row = max_row.max(character.input_coord.row);
            min_col = min_col.min(character.input_coord.column);
            max_col = max_col.max(character.input_coord.column);
        }
        // Mirrors `canvas.text_center_column` upstream.
        let text_center_column = min_col + (max_col - min_col) / 2;

        let canvas_top = terminal.canvas.height;
        let canvas_bottom = 1;

        for character in terminal.get_characters_mut() {
            // Style the character with its final gradient color (mapped by
            // row, bottom to top, as in the upstream vertical gradient
            // direction). This keeps SGR color codes on every rendered cell.
            let fraction = if max_row > min_row {
                (character.input_coord.row - min_row) as f64 / (max_row - min_row) as f64
            } else {
                1.0
            };
            let color = final_gradient
                .get_color_at_fraction(fraction)
                .expect("gradient spectrum is non-empty");
            character
                .animation
                .set_appearance(character.input_symbol, Some(ColorPair::fg_only(color)));

            // Python (vertical slice): characters left of the text center
            // start at Coord(column, canvas.top + 1); characters right of the
            // center start at Coord(column, canvas.bottom - 1). Both slide to
            // their input coordinates along an eased path.
            let start_row = if character.input_coord.column <= text_center_column {
                canvas_top + 1
            } else {
                canvas_bottom - 1
            };
            character.motion.current_coord =
                Coord::new(character.input_coord.column, start_row);

            let input_coord = character.input_coord;
            let path = character
                .motion
                .new_path("input_coord", MOVEMENT_SPEED, Some(easing::in_out_cubic));
            path.new_waypoint("input_coord", input_coord);
            character.motion.activate_path("input_coord");

            character.is_visible = true;
        }

        terminal.run(MAX_FRAMES)
    }
}
