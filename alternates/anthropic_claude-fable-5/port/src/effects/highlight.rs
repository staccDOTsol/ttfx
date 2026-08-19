//! Highlight effect: a specular highlight sweeps diagonally across the text
//! from the bottom-left to the top-right, briefly brightening each character
//! as it passes. Port of terminaltexteffects/effects/effect_highlight.py.
//!
//! Python defaults mirrored here:
//!   highlight_brightness = 1.75
//!   highlight_direction  = diagonal_bottom_left_to_top_right
//!   final_gradient_stops = 8A008A, 00D1FF, FFFFFF (vertical)

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

const FINAL_GRADIENT_STOPS: [&str; 3] = ["8A008A", "00D1FF", "FFFFFF"];
const FINAL_GRADIENT_STEPS: usize = 12;
const HIGHLIGHT_BRIGHTNESS: f64 = 1.75;
/// Steps used for the brighten/dim ramps of the highlight scene
/// (controls the apparent width of the highlight band).
const HIGHLIGHT_STEPS: usize = 5;
/// Ticks between successive diagonal groups being lit.
const GROUP_DELAY: u32 = 2;
const MAX_TICKS: u32 = 20_000;

pub struct Highlight;

impl Highlight {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Highlight {
    fn default() -> Self {
        Self::new()
    }
}

/// Brighten a color by multiplying its channels by `factor`, clamped to 255.
fn brighten(color: Color, factor: f64) -> Color {
    let scale = |v: u8| -> u8 { ((v as f64) * factor).round().clamp(0.0, 255.0) as u8 };
    Color::new(scale(color.r), scale(color.g), scale(color.b))
}

impl Effect for Highlight {
    fn name(&self) -> &str {
        "highlight"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());

        let stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .filter_map(|hex| Color::from_hex(hex))
            .collect();
        let final_gradient = Gradient::new(&stops, FINAL_GRADIENT_STEPS);
        let fallback_color = Color::new(255, 255, 255);
        let height = terminal.canvas.height;

        // The highlight sweeps along anti-diagonals: characters sharing the
        // same (column + row) light up together; smaller sums (bottom-left)
        // go first, larger sums (top-right) go last.
        let min_diag = terminal
            .get_characters()
            .iter()
            .map(|c| c.input_coord.column + c.input_coord.row)
            .min()
            .unwrap_or(0);

        // (activation_tick, character_id)
        let mut schedule: Vec<(u32, u32)> = Vec::new();

        for character in terminal.get_characters_mut() {
            let symbol = character.input_symbol;
            let coord = character.input_coord;

            // Final color from a vertical gradient over the canvas rows.
            let fraction = if height > 1 {
                (coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let base_color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(fallback_color);
            let bright_color = brighten(base_color, HIGHLIGHT_BRIGHTNESS);

            // Characters are visible in their final color from the start.
            character
                .animation
                .set_appearance(symbol, Some(ColorPair::fg_only(base_color)));
            character.is_visible = true;

            // Highlight scene: brighten to the highlight color, then dim
            // back down, ending on the base color.
            let ramp_up = Gradient::new(&[base_color, bright_color], HIGHLIGHT_STEPS);
            let ramp_down = Gradient::new(&[bright_color, base_color], HIGHLIGHT_STEPS);
            let scene = character.animation.new_scene("highlight", false);
            for &color in ramp_up.spectrum.iter().chain(ramp_down.spectrum.iter()) {
                scene.add_frame(symbol, 1, Some(ColorPair::fg_only(color)));
            }

            let diag = coord.column + coord.row;
            let activation_tick = ((diag - min_diag).max(0) as u32) * GROUP_DELAY;
            schedule.push((activation_tick, character.character_id));
        }

        schedule.sort_by_key(|&(tick, id)| (tick, id));

        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        let mut next_scheduled = 0usize;
        let mut tick: u32 = 0;
        while next_scheduled < schedule.len() || terminal.is_active() {
            // Activate the highlight scene for every character whose
            // diagonal group has been reached by the sweep.
            while next_scheduled < schedule.len() && schedule[next_scheduled].0 <= tick {
                let id = schedule[next_scheduled].1;
                if let Some(character) = terminal
                    .get_characters_mut()
                    .iter_mut()
                    .find(|c| c.character_id == id)
                {
                    character.animation.activate_scene("highlight");
                }
                next_scheduled += 1;
            }

            terminal.tick();
            frames.push(terminal.render_frame());

            tick += 1;
            if tick > MAX_TICKS {
                break;
            }
        }

        frames
    }
}
