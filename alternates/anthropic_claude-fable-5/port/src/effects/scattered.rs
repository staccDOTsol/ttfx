//! Scattered effect: the text is scattered across the canvas and moves into
//! position, flashing through a white-to-final-color gradient on arrival.
//!
//! Port of terminaltexteffects/effects/effect_scattered.py:
//!   - every character starts at a random coordinate on the canvas
//!   - each moves along an eased path (speed 0.5) back to its input coordinate
//!   - when its path completes, a gradient scene runs from white to the
//!     character's final color (taken from a vertical gradient across the
//!     canvas built from the default stops ff9048 -> ab9dff -> bdffea)

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const FINAL_GRADIENT_STOPS: [&str; 3] = ["ff9048", "ab9dff", "bdffea"];
const FINAL_GRADIENT_STEPS: usize = 12;
const FINAL_GRADIENT_FRAMES: u32 = 12;
const MOVEMENT_SPEED: f64 = 0.5;
const MAX_FRAMES: usize = 5000;

/// Small deterministic xorshift PRNG (the crate has no rand dependency).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Inclusive range [lo, hi], mirroring Python's random.randint.
    fn randint(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u64;
        lo + (self.next_u64() % span) as i32
    }
}

pub struct Scattered;

impl Scattered {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Scattered {
    fn name(&self) -> &str {
        "scattered"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let mut rng = Rng::new(0x5EED_C0DE_5CA7_7E4D);

        let stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .filter_map(|hex| Color::from_hex(hex))
            .collect();
        let final_gradient = Gradient::new(&stops, FINAL_GRADIENT_STEPS);

        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        let white = Color::new(255, 255, 255);

        for character in terminal.get_characters_mut() {
            // Vertical gradient mapping: bottom row -> first stop, top row -> last stop.
            let fraction = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                1.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(white);

            // Scatter: start at a random coordinate on the canvas
            // (degenerate canvases collapse to (1, 1), as upstream does).
            let start_coord = if width < 2 || height < 2 {
                Coord::new(1, 1)
            } else {
                Coord::new(rng.randint(1, width), rng.randint(1, height))
            };
            character.motion.current_coord = start_coord;

            // Path back home at the scattered movement speed with easing.
            let input_coord = character.input_coord;
            let path = character
                .motion
                .new_path("input_coord", MOVEMENT_SPEED, Some(easing::in_out_cubic));
            path.new_waypoint("input_coord", input_coord);
            character.motion.activate_path("input_coord");

            // Characters travel styled white until the gradient scene fires.
            character
                .animation
                .set_appearance(character.input_symbol, Some(ColorPair::fg_only(white)));

            // Gradient scene: white -> final color, one frame per spectrum
            // color, each held for FINAL_GRADIENT_FRAMES ticks.
            let char_gradient = Gradient::new(&[white, final_color], 10);
            let symbol = character.input_symbol;
            let scene = character.animation.new_scene("gradient", false);
            for color in &char_gradient.spectrum {
                scene.add_frame(symbol, FINAL_GRADIENT_FRAMES, Some(ColorPair::fg_only(*color)));
            }

            character.is_visible = true;
        }

        // Custom run loop so we can emulate the Python PATH_COMPLETE ->
        // ACTIVATE_SCENE event: once a character reaches home, its gradient
        // scene is activated.
        let mut gradient_started = vec![false; terminal.get_characters().len()];
        let mut frames = Vec::new();
        frames.push(terminal.render_frame());
        let mut count = 0;
        while terminal.is_active() && count < MAX_FRAMES {
            terminal.tick();
            for (index, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                if !gradient_started[index] && character.motion.movement_is_complete() {
                    gradient_started[index] = true;
                    character.animation.activate_scene("gradient");
                }
            }
            frames.push(terminal.render_frame());
            count += 1;
        }
        frames
    }
}
