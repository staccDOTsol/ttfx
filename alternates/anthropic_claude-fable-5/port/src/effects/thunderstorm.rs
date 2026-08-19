//! Thunderstorm effect: characters fall from the top of the canvas like rain,
//! with periodic lightning flashes that illuminate the whole screen, before
//! settling into their final gradient-colored resting positions.
//!
//! (Simplified port of terminaltexteffects/effects/effect_thunderstorm.py.)

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Small deterministic xorshift PRNG (the engine has no rng module).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    /// Uniform value in `[lo, hi)`; returns `lo` when the range is empty.
    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        if hi <= lo {
            return lo;
        }
        lo + self.next_u32() % (hi - lo)
    }

    fn f64(&mut self) -> f64 {
        self.next_u32() as f64 / u32::MAX as f64
    }
}

struct Drop {
    start_tick: u32,
    final_color: Color,
}

pub struct Thunderstorm;

impl Thunderstorm {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Thunderstorm {
    fn name(&self) -> &str {
        "thunderstorm"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;
        let mut rng = Rng::new(0x5EED_7457_0121_ABCD);

        // Rain / lightning palette (mirroring the Python defaults in spirit).
        let rain_symbols = ['o', '.', ',', '*', '|'];
        let rain_colors: Vec<Color> = ["31a0d4", "3f8fbf", "406e8e", "89b2cb"]
            .iter()
            .filter_map(|h| Color::from_hex(h))
            .collect();
        let lightning_color = Color::from_hex("ffff33").expect("valid hex");
        let final_gradient = Gradient::new(
            &[
                Color::from_hex("8a008a").expect("valid hex"),
                Color::from_hex("00d1ff").expect("valid hex"),
                Color::from_hex("ffffff").expect("valid hex"),
            ],
            12,
        );

        // Per-character drop scheduling and final resting colors.
        let mut drops: Vec<Drop> = Vec::new();
        {
            let chars = terminal.get_characters();
            let n = chars.len().max(1) as u32;
            let spread = (n * 2).clamp(20, 400);
            for character in chars {
                let frac = if height > 1 {
                    (character.input_coord.row - 1) as f64 / (height - 1) as f64
                } else {
                    0.0
                };
                let final_color = final_gradient
                    .get_color_at_fraction(frac)
                    .unwrap_or(lightning_color);
                drops.push(Drop {
                    start_tick: rng.range(0, spread),
                    final_color,
                });
            }
        }
        let max_start = drops.iter().map(|d| d.start_tick).max().unwrap_or(0);

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let mut flash_remaining: u32 = 0;
        let mut settle_ticks: u32 = 0;
        let max_frames: usize = 1200;
        let mut t: u32 = 0;

        while frames.len() < max_frames {
            // Launch raindrops whose start time has arrived.
            for (idx, drop) in drops.iter().enumerate() {
                if drop.start_tick == t {
                    let speed = 0.5 + rng.f64() * 0.7;
                    let character = &mut terminal.get_characters_mut()[idx];
                    let target = character.input_coord;
                    character.motion.current_coord = Coord::new(target.column, height);
                    let path = character.motion.new_path("fall", speed, Some(easing::in_quad));
                    path.new_waypoint("input", target);
                    character.motion.activate_path("fall");
                    character.is_visible = true;
                }
            }

            // Advance motion.
            terminal.tick();

            // Lightning flash bookkeeping.
            if flash_remaining > 0 {
                flash_remaining -= 1;
            } else if t > 5 && rng.f64() < 0.04 {
                flash_remaining = 2;
            }
            let flashing = flash_remaining > 0;

            // Style every visible character for this frame.
            for character in terminal.get_characters_mut() {
                if !character.is_visible {
                    continue;
                }
                let falling = !character.motion.movement_is_complete();
                let id = character.character_id as usize;
                if flashing {
                    let symbol = if falling {
                        rain_symbols[(t as usize + id) % rain_symbols.len()]
                    } else {
                        character.input_symbol
                    };
                    character
                        .animation
                        .set_appearance(symbol, Some(ColorPair::fg_only(lightning_color)));
                } else if falling {
                    let symbol = rain_symbols[(t as usize + id) % rain_symbols.len()];
                    let color = rain_colors[id % rain_colors.len()];
                    character
                        .animation
                        .set_appearance(symbol, Some(ColorPair::fg_only(color)));
                } else {
                    let color = drops[id].final_color;
                    character
                        .animation
                        .set_appearance(character.input_symbol, Some(ColorPair::fg_only(color)));
                }
            }

            frames.push(terminal.render_frame());

            // Once everything has launched and landed (and the sky is calm),
            // hold a few settled frames and finish.
            if t >= max_start && !terminal.is_active() && flash_remaining == 0 {
                settle_ticks += 1;
                if settle_ticks >= 3 {
                    break;
                }
            }

            t += 1;
        }

        frames
    }
}
