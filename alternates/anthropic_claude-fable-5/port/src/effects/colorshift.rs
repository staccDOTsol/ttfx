//! Colorshift: display a gradient that shifts colors across the text before
//! settling into a final gradient.
//!
//! Port of `terminaltexteffects/effects/effect_colorshift.py`. Each character
//! gets a scene that cycles through a looping gradient spectrum (optionally
//! offset by position so the colors appear to travel across the canvas), then
//! fades into its final color taken from the final gradient, mapped vertically.

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// The colorshift effect and its configuration (mirrors ColorShiftConfig).
pub struct Colorshift {
    /// Gradient stops cycled during the shift phase (rainbow by default).
    gradient_stops: Vec<Color>,
    /// Interpolation steps between shift gradient stops.
    gradient_steps: usize,
    /// Ticks each shift color is held.
    gradient_frames: u32,
    /// Number of full passes through the shift spectrum.
    cycles: u32,
    /// Offset the spectrum per character so the colors travel across the text.
    travel: bool,
    /// Reverse the travel direction.
    reverse_travel_direction: bool,
    /// Gradient used for the final, settled colors.
    final_gradient_stops: Vec<Color>,
    /// Interpolation steps between final gradient stops.
    final_gradient_steps: usize,
    /// Ticks each fade-to-final frame is held.
    final_gradient_frames: u32,
}

impl Colorshift {
    pub fn new() -> Self {
        let hex = |h: &str| Color::from_hex(h).expect("valid hex literal");
        Self {
            // Rainbow spectrum, as in the Python default config.
            gradient_stops: vec![
                hex("e81416"),
                hex("ffa500"),
                hex("faeb36"),
                hex("79c314"),
                hex("487de7"),
                hex("4b369d"),
                hex("70369d"),
            ],
            gradient_steps: 6,
            gradient_frames: 2,
            cycles: 2,
            travel: true,
            reverse_travel_direction: false,
            final_gradient_stops: vec![hex("8A008A"), hex("00D1FF"), hex("FFFFFF")],
            final_gradient_steps: 12,
            final_gradient_frames: 3,
        }
    }
}

impl Default for Colorshift {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Colorshift {
    fn name(&self) -> &str {
        "colorshift"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;

        // Build a looping gradient by appending the first stop, so the cycle
        // wraps smoothly back to its starting color (Python: Gradient(loop=True)).
        let mut loop_stops = self.gradient_stops.clone();
        if let Some(first) = self.gradient_stops.first().copied() {
            if self.gradient_stops.len() > 1 {
                loop_stops.push(first);
            }
        }
        let shift_gradient = Gradient::new(&loop_stops, self.gradient_steps);
        let spectrum = shift_gradient.spectrum.clone();
        let spectrum_len = spectrum.len().max(1);

        let final_gradient = Gradient::new(&self.final_gradient_stops, self.final_gradient_steps);

        for character in terminal.get_characters_mut() {
            // Diagonal travel: offset the spectrum by column + row so the
            // color wave sweeps across the canvas.
            let offset = if self.travel && !spectrum.is_empty() {
                let base =
                    (character.input_coord.column + character.input_coord.row).max(0) as usize;
                if self.reverse_travel_direction {
                    spectrum_len - (base % spectrum_len)
                } else {
                    base
                }
            } else {
                0
            };

            let symbol = character.input_symbol;
            let mut last_color = spectrum
                .get(offset % spectrum_len)
                .copied()
                .unwrap_or_else(|| Color::new(255, 255, 255));

            {
                let scene = character.animation.new_scene("colorshift", false);

                // Shift phase: cycle through the looping spectrum.
                for _ in 0..self.cycles {
                    for i in 0..spectrum_len {
                        let color = spectrum
                            .get((i + offset) % spectrum_len)
                            .copied()
                            .unwrap_or(last_color);
                        last_color = color;
                        scene.add_frame(symbol, self.gradient_frames, Some(ColorPair::fg_only(color)));
                    }
                }

                // Final phase: fade from the last shift color into the
                // character's final color (final gradient, vertical direction).
                let fraction = if height > 1 {
                    (character.input_coord.row - 1) as f64 / (height - 1) as f64
                } else {
                    0.0
                };
                let final_color = final_gradient
                    .get_color_at_fraction(fraction)
                    .unwrap_or(last_color);
                let fade_steps: usize = 10;
                for step in 1..=fade_steps {
                    let t = step as f64 / fade_steps as f64;
                    let color = Color::lerp(last_color, final_color, t);
                    scene.add_frame(
                        symbol,
                        self.final_gradient_frames,
                        Some(ColorPair::fg_only(color)),
                    );
                }
            }

            character.animation.activate_scene("colorshift");
            character.is_visible = true;
        }

        terminal.run(4000)
    }
}
