//! Blackhole effect: a group of characters forms a rotating black hole ring,
//! the remaining characters become a starfield that is pulled into the
//! singularity, then the hole collapses and explodes the text back into place.
//!
//! Port of terminaltexteffects/effects/effect_blackhole.py, adapted to the
//! simplified Rust engine (phases are orchestrated directly in `frames`).

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Small deterministic xorshift64 PRNG (the crate has no rand dependency).
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

    fn gen_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as usize
        }
    }

    /// Inclusive range.
    fn gen_range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            lo
        } else {
            lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i32
        }
    }

    fn gen_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.gen_f64()
    }
}

pub struct Blackhole;

impl Blackhole {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Blackhole {
    fn default() -> Self {
        Self::new()
    }
}

const STAR_SYMBOLS: [char; 16] = [
    '*', '✸', '✺', '✹', '✷', '✵', '✶', '⋆', '.', '⬫', '⬪', '⬩', '⬨', '⬧', '⬦', '⬥',
];

impl Effect for Blackhole {
    fn name(&self) -> &str {
        "blackhole"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let mut frames: Vec<String> = Vec::new();
        let mut rng = Rng::new(0x5EED_B14C_4801_E001);

        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        let center = terminal.canvas.center();

        let n = terminal.get_characters().len();
        if n == 0 {
            frames.push(terminal.render_frame());
            return frames;
        }

        // Colors mirroring the Python defaults.
        let blackhole_color = Color::from_hex("ffffff").expect("valid hex");
        let star_colors: Vec<Color> = ["ffcc0d", "ff7326", "ff194d", "bf2669", "702a8c", "049dbf"]
            .iter()
            .map(|h| Color::from_hex(h).expect("valid hex"))
            .collect();
        let final_stops = [
            Color::from_hex("8A008A").expect("valid hex"),
            Color::from_hex("00D1FF").expect("valid hex"),
            Color::from_hex("ffffff").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&final_stops, 12);

        // Final gradient color per character, mapped vertically by row
        // (as the Python effect's VERTICAL gradient direction does).
        let final_colors: Vec<Color> = terminal
            .get_characters()
            .iter()
            .map(|c| {
                let t = if height > 1 {
                    (c.input_coord.row - 1) as f64 / (height - 1) as f64
                } else {
                    0.0
                };
                final_gradient
                    .get_color_at_fraction(t)
                    .unwrap_or(blackhole_color)
            })
            .collect();

        // Choose round(sqrt(n)) characters to form the black hole ring.
        let mut indices: Vec<usize> = (0..n).collect();
        for i in (1..indices.len()).rev() {
            let j = rng.gen_usize(i + 1);
            indices.swap(i, j);
        }
        let bh_count = ((n as f64).sqrt().round() as usize).clamp(1, n);
        let bh: Vec<usize> = indices[..bh_count].to_vec();
        let stars: Vec<usize> = indices[bh_count..].to_vec();

        let blackhole_radius = (width.min(height) / 3).max(2);
        let ring: Vec<Coord> = find_coords_on_circle(center, blackhole_radius, bh_count);

        // ---- Phase 1: forming — black hole characters travel to the ring. ----
        {
            let chars = terminal.get_characters_mut();
            for (k, &i) in bh.iter().enumerate() {
                let ch = &mut chars[i];
                ch.is_visible = true;
                ch.animation
                    .set_appearance('*', Some(ColorPair::fg_only(blackhole_color)));
                {
                    let path = ch.motion.new_path("blackhole", 0.7, Some(easing::in_out_sine));
                    path.new_waypoint("0", ring[k]);
                }
                ch.motion.activate_path("blackhole");
            }
        }
        frames.push(terminal.render_frame());
        let mut guard = 0usize;
        while bh
            .iter()
            .any(|&i| !terminal.get_characters()[i].motion.movement_is_complete())
            && guard < 600
        {
            terminal.tick();
            frames.push(terminal.render_frame());
            guard += 1;
        }

        // ---- Phase 2: rotation + starfield consumption. ----
        {
            let chars = terminal.get_characters_mut();
            // Rotation waypoints: ring positions rotated to start at each
            // character's own position (as the Python effect builds them).
            for (k, &i) in bh.iter().enumerate() {
                let ch = &mut chars[i];
                {
                    let path = ch.motion.new_path("rotation", 0.45, None);
                    for (w, idx) in (k..bh_count).chain(0..k).enumerate() {
                        path.new_waypoint(&w.to_string(), ring[idx]);
                    }
                }
                ch.motion.activate_path("rotation");
            }
            // Scatter the remaining characters into a starfield and send each
            // toward the singularity with in_expo easing.
            for &i in &stars {
                let starfield_coord = Coord::new(
                    rng.gen_range_i32(1, width),
                    rng.gen_range_i32(1, height),
                );
                let symbol = STAR_SYMBOLS[rng.gen_usize(STAR_SYMBOLS.len())];
                let color = star_colors[rng.gen_usize(star_colors.len())];
                let speed = rng.uniform(0.17, 0.30);
                let ch = &mut chars[i];
                ch.motion.current_coord = starfield_coord;
                ch.animation
                    .set_appearance(symbol, Some(ColorPair::fg_only(color)));
                ch.is_visible = true;
                {
                    let path = ch.motion.new_path("singularity", speed, Some(easing::in_expo));
                    path.new_waypoint("0", center);
                }
                ch.motion.activate_path("singularity");
            }
        }
        frames.push(terminal.render_frame());

        let mut consumed: Vec<bool> = vec![false; n];
        guard = 0;
        while stars.iter().any(|&i| !consumed[i]) && guard < 1200 {
            terminal.tick();
            {
                let chars = terminal.get_characters_mut();
                for &i in &stars {
                    if !consumed[i] && chars[i].motion.movement_is_complete() {
                        consumed[i] = true;
                        chars[i].is_visible = false; // swallowed by the singularity
                    }
                }
                // Keep the ring rotating: reactivate the loop when it completes.
                for &i in &bh {
                    if chars[i].motion.movement_is_complete() {
                        chars[i].motion.activate_path("rotation");
                    }
                }
            }
            frames.push(terminal.render_frame());
            guard += 1;
        }
        // Force-consume any stragglers so the collapse can proceed.
        {
            let chars = terminal.get_characters_mut();
            for &i in &stars {
                if !consumed[i] {
                    chars[i].is_visible = false;
                }
            }
        }

        // ---- Phase 3: collapse — the ring falls into the center point. ----
        {
            let chars = terminal.get_characters_mut();
            for &i in &bh {
                let ch = &mut chars[i];
                {
                    let path = ch.motion.new_path("collapse", 0.4, Some(easing::in_expo));
                    path.new_waypoint("0", center);
                }
                ch.motion.activate_path("collapse");
            }
        }
        guard = 0;
        while bh
            .iter()
            .any(|&i| !terminal.get_characters()[i].motion.movement_is_complete())
            && guard < 400
        {
            terminal.tick();
            frames.push(terminal.render_frame());
            guard += 1;
        }

        // ---- Phase 4: explosion — everything bursts back to the input text. ----
        {
            let chars = terminal.get_characters_mut();
            for i in 0..n {
                let ch = &mut chars[i];
                let input_symbol = ch.input_symbol;
                let input_coord = ch.input_coord;
                ch.motion.current_coord = center;
                ch.is_visible = true;
                // Cooldown scene: white flash fading into the final gradient color.
                let cooldown = Gradient::new(&[blackhole_color, final_colors[i]], 10);
                {
                    let scene = ch.animation.new_scene("cooldown", false);
                    scene.frames.clear();
                    scene.add_frame('*', 2, Some(ColorPair::fg_only(blackhole_color)));
                    for color in &cooldown.spectrum {
                        scene.add_frame(input_symbol, 3, Some(ColorPair::fg_only(*color)));
                    }
                }
                ch.animation.activate_scene("cooldown");
                {
                    let path = ch.motion.new_path("explosion", 0.7, Some(easing::out_expo));
                    path.new_waypoint("0", input_coord);
                }
                ch.motion.activate_path("explosion");
            }
        }
        frames.push(terminal.render_frame());
        guard = 0;
        while terminal.is_active() && guard < 600 {
            terminal.tick();
            frames.push(terminal.render_frame());
            guard += 1;
        }

        // Final settle: every character at home with its final gradient color.
        {
            let chars = terminal.get_characters_mut();
            for i in 0..n {
                let ch = &mut chars[i];
                let input_symbol = ch.input_symbol;
                ch.motion.current_coord = ch.input_coord;
                ch.animation
                    .set_appearance(input_symbol, Some(ColorPair::fg_only(final_colors[i])));
                ch.is_visible = true;
            }
        }
        frames.push(terminal.render_frame());

        frames
    }
}
