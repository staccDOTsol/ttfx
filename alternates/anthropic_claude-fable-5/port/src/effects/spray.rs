//! Spray effect (port of terminaltexteffects/effects/effect_spray.py).
//!
//! Characters are sprayed from a single point on the canvas edge toward their
//! input coordinates at varying speeds. Each droplet cycles through a short
//! highlight gradient before settling on its final gradient color.

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing::{self, EasingFunction};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Default final gradient stops from the Python original.
const FINAL_GRADIENT_STOPS: [&str; 3] = ["8A008A", "00D1FF", "FFFFFF"];
const FINAL_GRADIENT_STEPS: usize = 12;
/// Highlight color applied to characters while they are in flight.
const HIGHLIGHT_COLOR: &str = "88F7E2";
/// Fraction of the character count released (at most) per tick.
const SPRAY_VOLUME: f64 = 0.005;
/// Movement speed range (cells per tick), as in the Python defaults.
const MOVEMENT_SPEED: (f64, f64) = (0.4, 1.0);
/// Droplet gradient steps between highlight and final color.
const DROPLET_GRADIENT_STEPS: usize = 7;
/// Ticks each droplet gradient frame is held.
const DROPLET_FRAME_DURATION: u32 = 3;
/// Safety bound on total rendered frames.
const MAX_FRAMES: usize = 5000;
/// Fixed seed so effect output is deterministic run-to-run.
const SEED: u64 = 0x5EED_5EED_1234_ABCD;

/// Small deterministic PRNG (LCG) standing in for Python's `random` module.
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

    /// Uniform float in `[lo, hi)`.
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * (self.next_u32() as f64 / (u32::MAX as f64 + 1.0))
    }

    /// Uniform integer in `[lo, hi]` (inclusive), like Python's `randint`.
    fn randint(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        lo + self.next_u32() % (hi - lo + 1)
    }

    /// Fisher-Yates shuffle, like Python's `random.shuffle`.
    fn shuffle<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }
        for i in (1..items.len()).rev() {
            let j = self.randint(0, i as u32) as usize;
            items.swap(i, j);
        }
    }
}

/// The spray effect.
pub struct Spray;

impl Spray {
    pub fn new() -> Self {
        Spray
    }
}

impl Effect for Spray {
    fn name(&self) -> &str {
        "spray"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let mut rng = Lcg::new(SEED);

        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        // Default spray position "e": the east edge at the vertical center,
        // matching the Python spray_position default.
        let spray_origin = Coord::new(width, (height + 1) / 2);

        let stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .filter_map(|hex| Color::from_hex(hex))
            .collect();
        let final_gradient = Gradient::new(&stops, FINAL_GRADIENT_STEPS);
        let highlight =
            Color::from_hex(HIGHLIGHT_COLOR).unwrap_or(Color::new(0x88, 0xF7, 0xE2));

        // --- build(): place every character at the spray origin, give it a
        // path back to its input coordinate and a droplet gradient scene. ---
        let mut pending: Vec<u32> = Vec::new();
        for character in terminal.get_characters_mut() {
            // Vertical final-gradient direction: color chosen by row fraction.
            let fraction = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(highlight);

            character.motion.current_coord = spray_origin;

            let speed = rng.uniform(MOVEMENT_SPEED.0, MOVEMENT_SPEED.1);
            let path = character.motion.new_path(
                "input_coord",
                speed,
                Some(easing::out_expo as EasingFunction),
            );
            path.new_waypoint("input_coord", character.input_coord);

            // Droplet scene: highlight -> final color; the scene's last frame
            // (the final gradient color) persists once the scene completes,
            // so settled characters remain styled.
            let symbol = character.input_symbol;
            let droplet_gradient =
                Gradient::new(&[highlight, final_color], DROPLET_GRADIENT_STEPS);
            let scene = character.animation.new_scene("droplet", false);
            for color in &droplet_gradient.spectrum {
                scene.add_frame(
                    symbol,
                    DROPLET_FRAME_DURATION,
                    Some(ColorPair::fg_only(*color)),
                );
            }

            pending.push(character.character_id);
        }
        rng.shuffle(&mut pending);

        let volume = ((pending.len() as f64 * SPRAY_VOLUME) as u32).max(1);

        // --- frame loop: release a random number of droplets each tick. ---
        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        while (!pending.is_empty() || terminal.is_active()) && frames.len() < MAX_FRAMES {
            if !pending.is_empty() {
                let release_count = rng.randint(1, volume) as usize;
                for _ in 0..release_count {
                    let Some(id) = pending.pop() else {
                        break;
                    };
                    if let Some(character) = terminal
                        .get_characters_mut()
                        .iter_mut()
                        .find(|c| c.character_id == id)
                    {
                        character.is_visible = true;
                        character.animation.activate_scene("droplet");
                        character.motion.activate_path("input_coord");
                    }
                }
            }
            terminal.tick();
            frames.push(terminal.render_frame());
        }

        frames
    }
}
