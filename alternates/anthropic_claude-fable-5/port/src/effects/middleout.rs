//! Middleout effect: text expands from the center row (or column) outward,
//! then travels to its final position while fading to the final gradient.
//! Port of terminaltexteffects/effects/effect_middleout.py.

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Maximum number of frames to render (safety bound).
const MAX_FRAMES: usize = 5000;

/// Middleout effect configuration, mirroring the Python `MiddleOutConfig` defaults.
pub struct Middleout {
    starting_color: Color,
    final_gradient_stops: Vec<Color>,
    final_gradient_steps: usize,
    /// "vertical" expands out from the center row; false = horizontal.
    expand_direction_vertical: bool,
    center_movement_speed: f64,
    full_movement_speed: f64,
    center_easing: easing::EasingFunction,
    full_easing: easing::EasingFunction,
}

impl Middleout {
    pub fn new() -> Self {
        Self {
            starting_color: Color::from_hex("ffffff").expect("valid hex"),
            final_gradient_stops: vec![
                Color::from_hex("8A008A").expect("valid hex"),
                Color::from_hex("00D1FF").expect("valid hex"),
                Color::from_hex("FFFFFF").expect("valid hex"),
            ],
            final_gradient_steps: 12,
            expand_direction_vertical: true,
            center_movement_speed: 0.35,
            full_movement_speed: 0.35,
            center_easing: easing::in_out_sine,
            full_easing: easing::in_out_sine,
        }
    }
}

impl Default for Middleout {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Middleout {
    fn name(&self) -> &str {
        "middleout"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let center = terminal.canvas.center();
        let height = terminal.canvas.height;

        let final_gradient = Gradient::new(&self.final_gradient_stops, self.final_gradient_steps);
        let starting_color = self.starting_color;
        let expand_vertical = self.expand_direction_vertical;
        let center_speed = self.center_movement_speed;
        let full_speed = self.full_movement_speed;
        let center_easing = self.center_easing;
        let full_easing = self.full_easing;

        // ---- build (mirrors Python build()) ----
        for character in terminal.get_characters_mut() {
            // Final gradient color mapped by row (vertical gradient direction, the
            // upstream default), matching build_coordinate_color_mapping.
            let t = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(t)
                .unwrap_or(starting_color);

            // Characters start at the canvas center.
            character.motion.current_coord = center;

            // Center waypoint: keep own column (vertical expand) or own row (horizontal).
            let (column, row) = if expand_vertical {
                (character.input_coord.column, center.row)
            } else {
                (center.column, character.input_coord.row)
            };
            {
                let center_path =
                    character
                        .motion
                        .new_path("center", center_speed, Some(center_easing));
                center_path.new_waypoint("0", Coord::new(column, row));
            }
            {
                let full_path = character
                    .motion
                    .new_path("full", full_speed, Some(full_easing));
                full_path.new_waypoint("0", character.input_coord);
            }

            // "full" scene: gradient from the starting color to the character's
            // final gradient color (Python: apply_gradient_to_symbols, 10 steps).
            let char_gradient = Gradient::new(&[starting_color, final_color], 10);
            let input_symbol = character.input_symbol;
            {
                let full_scene = character.animation.new_scene("full", false);
                for color in &char_gradient.spectrum {
                    full_scene.add_frame(input_symbol, 10, Some(ColorPair::fg_only(*color)));
                }
            }

            // Initial appearance: input symbol in the starting color (styled output).
            character
                .animation
                .set_appearance(input_symbol, Some(ColorPair::fg_only(starting_color)));

            character.motion.activate_path("center");
            character.is_visible = true;
        }

        // ---- frame loop (mirrors Python __next__) ----
        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        // Phase 1: everyone converges on the center row/column.
        while frames.len() < MAX_FRAMES
            && terminal
                .get_characters()
                .iter()
                .any(|c| !c.motion.movement_is_complete())
        {
            terminal.tick();
            frames.push(terminal.render_frame());
        }

        // Phase 2: once expanded, activate the "full" path and the "full" scene
        // (upstream registers PATH_ACTIVATED -> ACTIVATE_SCENE for the full path).
        for character in terminal.get_characters_mut() {
            character.motion.activate_path("full");
            character.animation.activate_scene("full");
        }

        while frames.len() < MAX_FRAMES && terminal.is_active() {
            terminal.tick();
            frames.push(terminal.render_frame());
        }

        frames
    }
}
