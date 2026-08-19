//! Unstable effect: characters spawn jumbled, rumble with increasing intensity,
//! explode to the canvas edges, then reassemble into the input text.
//!
//! Port of terminaltexteffects/effects/effect_unstable.py.

use super::Effect;
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use crate::engine::terminal::{Terminal, TerminalConfig};

/// Small deterministic PRNG (LCG) so the effect needs no external crates.
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

    /// Inclusive range [lo, hi], like Python's random.randint.
    fn randint(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() % ((hi - lo + 1) as u32)) as i32
    }

    fn choice(&mut self, options: &[i32]) -> i32 {
        options[(self.next_u32() as usize) % options.len()]
    }
}

pub struct Unstable;

impl Unstable {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Unstable {
    fn name(&self) -> &str {
        "unstable"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        let mut rng = Lcg::new(0x7f4a_7c15_9e37_79b9);

        if terminal.get_characters().is_empty() {
            return vec![terminal.render_frame()];
        }

        // Config mirrored from the Python defaults.
        let unstable_color = Color::from_hex("ff9200").expect("valid hex");
        let start_color = Color::from_hex("ffffff").expect("valid hex");
        let explosion_speed = 0.75;
        let reassembly_speed = 0.75;
        let final_gradient_stops = [
            Color::from_hex("8A008A").expect("valid hex"),
            Color::from_hex("00D1FF").expect("valid hex"),
            Color::from_hex("FFFFFF").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&final_gradient_stops, 12);

        // Pool of input coordinates to jumble the characters across the text area.
        let mut character_coords: Vec<Coord> = terminal
            .get_characters()
            .iter()
            .map(|c| c.input_coord)
            .collect();
        // Jumbled coordinate per character, in arena (input) order.
        let mut jumbled_coords: Vec<Coord> = Vec::with_capacity(character_coords.len());

        // --- build (mirrors UnstableIterator.build) ---
        for character in terminal.get_characters_mut() {
            let input_coord = character.input_coord;
            let input_symbol = character.input_symbol;

            // Pick an explosion target on a random canvas edge.
            let pos = rng.randint(0, 3);
            let (col, row) = match pos {
                0 => (1, rng.randint(1, height)),
                1 => (width, rng.randint(1, height)),
                2 => (rng.randint(1, width), 1),
                _ => (rng.randint(1, width), height),
            };

            // Assign a random jumbled starting coordinate from the pool.
            let idx = rng.randint(0, character_coords.len() as i32 - 1) as usize;
            let jumbled_coord = character_coords.remove(idx);
            jumbled_coords.push(jumbled_coord);
            character.motion.current_coord = jumbled_coord;

            // Explosion path to the canvas edge.
            let explosion_path =
                character
                    .motion
                    .new_path("explosion", explosion_speed, Some(easing::out_expo));
            explosion_path.new_waypoint("0", Coord::new(col, row));

            // Reassembly path back to the original input coordinate.
            let reassembly_path =
                character
                    .motion
                    .new_path("reassembly", reassembly_speed, Some(easing::out_expo));
            reassembly_path.new_waypoint("0", input_coord);

            // Rumble scene: fade from the neutral start color to the unstable color.
            let rumble_gradient = Gradient::new(&[start_color, unstable_color], 12);
            let rumble_scn = character.animation.new_scene("rumble", false);
            for color in &rumble_gradient.spectrum {
                rumble_scn.add_frame(input_symbol, 6, Some(ColorPair::fg_only(*color)));
            }

            // Final scene: fade from unstable color to the per-character final
            // gradient color (vertical gradient across the canvas).
            let fraction = if height > 1 {
                (input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(start_color);
            let settle_gradient = Gradient::new(&[unstable_color, final_color], 12);
            let final_scn = character.animation.new_scene("final", false);
            for color in &settle_gradient.spectrum {
                final_scn.add_frame(input_symbol, 5, Some(ColorPair::fg_only(*color)));
            }

            character.animation.activate_scene("rumble");
            character.is_visible = true;
        }

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        // --- phase 1: rumble (accelerating jitter around the jumbled coords) ---
        let rumble_ticks: i32 = 100;
        for tick in 0..rumble_ticks {
            let period = ((rumble_ticks - tick) / 10 + 1).max(1);
            if tick % period == 0 {
                for (i, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                    let column_offset = rng.choice(&[-1, 0, 1]);
                    let row_offset = rng.choice(&[-1, 0, 1]);
                    character.motion.current_coord = Coord::new(
                        jumbled_coords[i].column + column_offset,
                        jumbled_coords[i].row + row_offset,
                    );
                }
            }
            terminal.tick();
            frames.push(terminal.render_frame());
        }
        // Snap back onto the jumbled coordinates before exploding.
        for (i, character) in terminal.get_characters_mut().iter_mut().enumerate() {
            character.motion.current_coord = jumbled_coords[i];
        }

        // --- phase 2: explosion (fly to the canvas edges) ---
        for character in terminal.get_characters_mut() {
            character.motion.activate_path("explosion");
        }
        let mut guard = 0usize;
        while terminal
            .get_characters()
            .iter()
            .any(|c| !c.motion.movement_is_complete())
            && guard < 600
        {
            terminal.tick();
            frames.push(terminal.render_frame());
            guard += 1;
        }

        // --- phase 3: reassembly (return home, settle into final colors) ---
        for character in terminal.get_characters_mut() {
            character.animation.activate_scene("final");
            character.motion.activate_path("reassembly");
        }
        let mut guard = 0usize;
        while terminal.is_active() && guard < 800 {
            terminal.tick();
            frames.push(terminal.render_frame());
            guard += 1;
        }
        frames.push(terminal.render_frame());

        frames
    }
}
