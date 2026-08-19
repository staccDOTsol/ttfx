//! Smoke effect: characters rise from the bottom of the canvas as drifting
//! wisps of smoke, wandering upward along jittered paths while cycling
//! through smoke symbols and a gray gradient, before condensing into their
//! final, gradient-colored form at their input coordinates.

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_length_of_line, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Small deterministic PRNG (LCG) so the effect is reproducible without an
/// external rng module.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed | 1)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    /// Inclusive range [lo, hi].
    fn gen_range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u32;
        lo + (self.next_u32() % span) as i32
    }

    fn gen_f64(&mut self) -> f64 {
        self.next_u32() as f64 / u32::MAX as f64
    }

    fn choice<T: Copy>(&mut self, items: &[T]) -> T {
        items[(self.next_u32() as usize) % items.len()]
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }
        for i in (1..items.len()).rev() {
            let j = (self.next_u32() as usize) % (i + 1);
            items.swap(i, j);
        }
    }
}

pub struct Smoke;

impl Smoke {
    pub fn new() -> Self {
        Smoke
    }
}

const SMOKE_SYMBOLS: [char; 10] = ['\'', '.', ':', ';', '~', '*', '░', '▒', '▓', '░'];

impl Effect for Smoke {
    fn name(&self) -> &str {
        "smoke"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        let mut rng = Lcg::new(0x5EED_C0DE_5EED_C0DE);

        // Final colors: a vertical gradient across the canvas rows.
        let final_gradient = Gradient::new(
            &[
                Color::from_hex("8A008A").expect("valid hex"),
                Color::from_hex("00D1FF").expect("valid hex"),
                Color::from_hex("FFFFFF").expect("valid hex"),
            ],
            12,
        );
        // Smoke colors: dark gray thickening into pale gray.
        let smoke_gradient = Gradient::new(
            &[
                Color::from_hex("434343").expect("valid hex"),
                Color::from_hex("767676").expect("valid hex"),
                Color::from_hex("B2B2B2").expect("valid hex"),
            ],
            8,
        );
        let fallback_gray = Color::new(120, 120, 120);
        let fallback_final = Color::new(255, 255, 255);

        // Configure each character: a wandering upward path plus a smoke scene.
        for character in terminal.get_characters_mut() {
            let input_coord = character.input_coord;
            let input_symbol = character.input_symbol;

            let row_fraction = if height > 1 {
                (input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(row_fraction)
                .unwrap_or(fallback_final);

            // Start below/at the bottom row, near the character's column.
            let start_col = (input_coord.column + rng.gen_range(-3, 3)).clamp(1, width);
            let start_coord = Coord::new(start_col, 1);
            character.motion.current_coord = start_coord;

            let speed = 0.3 + rng.gen_f64() * 0.35;
            let mut est_distance = 0.0;
            {
                let path = character.motion.new_path("rise", speed, Some(easing::out_sine));
                let mut prev = start_coord;
                let wander_points = 3;
                for i in 1..=wander_points {
                    let t = i as f64 / (wander_points + 1) as f64;
                    let row = (1.0 + (input_coord.row - 1) as f64 * t).round() as i32;
                    let row = row.clamp(1, height);
                    let col = (input_coord.column + rng.gen_range(-4, 4)).clamp(1, width);
                    let coord = Coord::new(col, row);
                    est_distance += find_length_of_line(prev, coord);
                    path.new_waypoint(&format!("wander_{i}"), coord);
                    prev = coord;
                }
                est_distance += find_length_of_line(prev, input_coord);
                path.new_waypoint("input_coord", input_coord);
            }

            // Size the smoke animation to roughly match the travel time.
            let est_steps = ((est_distance / speed).ceil() as u32).max(4);
            let frame_duration = 3u32;
            let smoke_frame_count = ((est_steps / frame_duration).max(4)).min(48) as usize;

            {
                let scene = character.animation.new_scene("smoke", false);
                for k in 0..smoke_frame_count {
                    let frac = if smoke_frame_count > 1 {
                        k as f64 / (smoke_frame_count - 1) as f64
                    } else {
                        0.0
                    };
                    let color = smoke_gradient
                        .get_color_at_fraction(frac)
                        .unwrap_or(fallback_gray);
                    let symbol = rng.choice(&SMOKE_SYMBOLS);
                    scene.add_frame(symbol, frame_duration, Some(ColorPair::fg_only(color)));
                }
                // Condense into the final, gradient-colored character.
                scene.add_frame(input_symbol, 1, Some(ColorPair::fg_only(final_color)));
            }
        }

        // Staggered activation: characters begin rising in shuffled order,
        // a few per tick, like wisps peeling off a smoldering source.
        let total = terminal.get_characters().len();
        let mut order: Vec<usize> = (0..total).collect();
        rng.shuffle(&mut order);

        let per_tick = (total / 40).max(1);
        let max_frames = 1200usize;

        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        let mut activated = 0usize;
        let mut tick_count = 0usize;
        loop {
            if activated < total {
                for _ in 0..per_tick {
                    if activated >= total {
                        break;
                    }
                    let idx = order[activated];
                    let characters = terminal.get_characters_mut();
                    let character = &mut characters[idx];
                    character.is_visible = true;
                    character.motion.activate_path("rise");
                    character.animation.activate_scene("smoke");
                    activated += 1;
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());
            tick_count += 1;

            if activated >= total && !terminal.is_active() {
                break;
            }
            if tick_count >= max_frames {
                break;
            }
        }

        frames
    }
}
