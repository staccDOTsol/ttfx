//! Expand effect: characters begin at the canvas center and expand outward
//! to their input coordinates, colored by a gradient synced over the motion.
//! Port of terminaltexteffects/effects/effect_expand.py.

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Expands the text from a single point at the canvas center.
pub struct Expand {
    movement_speed: f64,
    final_gradient_stops: Vec<Color>,
    final_gradient_steps: usize,
    final_gradient_frames: u32,
}

impl Expand {
    pub fn new() -> Self {
        Self {
            // Defaults mirror ExpandConfig in the Python original.
            movement_speed: 0.35,
            final_gradient_stops: vec![
                Color::from_hex("8A008A").expect("valid hex"),
                Color::from_hex("00D1FF").expect("valid hex"),
                Color::from_hex("FFFFFF").expect("valid hex"),
            ],
            final_gradient_steps: 12,
            final_gradient_frames: 5,
        }
    }
}

impl Default for Expand {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Expand {
    fn name(&self) -> &str {
        "expand"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());

        // Build the final gradient and map each character's final color by
        // its row (vertical gradient direction, as upstream does).
        let final_gradient = Gradient::new(&self.final_gradient_stops, self.final_gradient_steps);
        let start_color = final_gradient
            .spectrum
            .first()
            .copied()
            .unwrap_or(Color::new(255, 255, 255));

        let height = terminal.config.height.max(1);
        let center = terminal.canvas.center();
        let gradient_frames = self.final_gradient_frames;
        let movement_speed = self.movement_speed;

        for character in terminal.get_characters_mut() {
            // Vertical fraction of this character's home row on the canvas.
            let fraction = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(start_color);

            // Start every character at the canvas center and send it home.
            character.motion.current_coord = center;
            {
                let input_coord_path = character.motion.new_path(
                    "input_coord",
                    movement_speed,
                    Some(easing::in_out_quad),
                );
                input_coord_path.new_waypoint("input_coord", character.input_coord);
            }
            character.motion.activate_path("input_coord");

            // Gradient scene: ramp from the gradient's first stop to this
            // character's final color while it travels.
            {
                let gradient_scn = character.animation.new_scene("expand_gradient", false);
                let ramp = Gradient::new(&[start_color, final_color], 10);
                for color in &ramp.spectrum {
                    gradient_scn.add_frame(
                        character.input_symbol,
                        gradient_frames,
                        Some(ColorPair::fg_only(*color)),
                    );
                }
            }
            character.animation.activate_scene("expand_gradient");

            // Ensure the very first rendered frame is already styled.
            character
                .animation
                .set_appearance(character.input_symbol, Some(ColorPair::fg_only(start_color)));

            character.is_visible = true;
        }

        terminal.run(2000)
    }
}
