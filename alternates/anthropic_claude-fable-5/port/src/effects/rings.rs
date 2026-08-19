//! Rings effect: characters are assembled into concentric rings around the
//! canvas center, spin, disperse, spin again, then return home while fading
//! into the final gradient. Port of terminaltexteffects/effects/effect_rings.py
//! adapted to the simplified engine in this crate.

use std::collections::HashSet;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

const MAX_FRAMES: usize = 1200;
const ASSEMBLE_TICK_CAP: usize = 400;
const SPIN_TICKS: usize = 70;
const DISPERSE_TICKS: usize = 50;
const SPIN_DISPERSE_CYCLES: usize = 2;

/// Small deterministic PRNG (the crate has no rng module yet).
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) as u32
    }

    /// Inclusive range.
    fn gen_range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() % ((hi - lo + 1) as u32)) as i32
    }

    fn shuffle<T>(&mut self, v: &mut [T]) {
        if v.len() < 2 {
            return;
        }
        for i in (1..v.len()).rev() {
            let j = (self.next_u32() as usize) % (i + 1);
            v.swap(i, j);
        }
    }
}

struct RingData {
    coords: Vec<Coord>,
    color: Color,
}

struct Member {
    idx: usize,
    ring: usize,
    point: usize,
}

pub struct Rings;

impl Rings {
    pub fn new() -> Self {
        Rings
    }
}

impl Effect for Rings {
    fn name(&self) -> &str {
        "rings"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.config.width;
        let height = terminal.config.height;
        let center = terminal.canvas.center();
        let char_count = terminal.get_characters().len();
        if char_count == 0 {
            return vec![String::new()];
        }

        let mut rng = Lcg::new(0x5eed ^ ((char_count as u64) << 1));

        // Colors mirroring the Python defaults.
        let ring_colors = [
            Color::from_hex("ab48ff").expect("valid hex"),
            Color::from_hex("e7b2b2").expect("valid hex"),
            Color::from_hex("fffebd").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(
            &[
                Color::from_hex("8A008A").expect("valid hex"),
                Color::from_hex("00D1FF").expect("valid hex"),
                Color::from_hex("FFFFFF").expect("valid hex"),
            ],
            12,
        );
        let gray = Color::new(0x66, 0x66, 0x66);

        // Ring geometry (Python: ring_gap = round(min(canvas dims) * 0.1)).
        let ring_gap = (((width.min(height)) as f64) * 0.1).round().max(1.0) as i32;
        let max_radius = width.min(height) / 2 - 1;

        let mut rings: Vec<RingData> = Vec::new();
        let mut radius = ring_gap;
        while radius <= max_radius {
            let raw = find_coords_on_circle(center, radius, (7 * radius).max(1) as usize);
            let mut seen: HashSet<Coord> = HashSet::new();
            let coords: Vec<Coord> = raw.into_iter().filter(|c| seen.insert(*c)).collect();
            if !coords.is_empty() {
                let color = ring_colors[rings.len() % ring_colors.len()];
                rings.push(RingData { coords, color });
            }
            radius += ring_gap;
        }

        // Assign shuffled characters to ring points (inner rings first).
        let mut pending: Vec<usize> = (0..char_count).collect();
        rng.shuffle(&mut pending);
        let mut members: Vec<Member> = Vec::new();
        'assign: for (ri, ring) in rings.iter().enumerate() {
            for pi in 0..ring.coords.len() {
                match pending.pop() {
                    Some(idx) => members.push(Member { idx, ring: ri, point: pi }),
                    None => break 'assign,
                }
            }
        }
        let externals: Vec<usize> = pending;

        // Per-character start colors from the final gradient, mapped by row.
        let infos: Vec<(char, Coord)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.input_symbol, c.input_coord))
            .collect();
        let start_color_for = |coord: Coord| -> Color {
            let denom = (height - 1).max(1) as f64;
            let frac = (coord.row - 1) as f64 / denom;
            final_gradient.get_color_at_fraction(frac).unwrap_or(gray)
        };

        // ---- Setup: visibility, appearance, paths, scenes -------------------
        {
            let chars = terminal.get_characters_mut();
            for (i, ch) in chars.iter_mut().enumerate() {
                ch.is_visible = true;
                let (sym, coord) = infos[i];
                ch.animation
                    .set_appearance(sym, Some(ColorPair::fg_only(start_color_for(coord))));
            }
        }

        for m in &members {
            let ring = &rings[m.ring];
            let ring_coord = ring.coords[m.point];
            let n = ring.coords.len();
            let ring_color = ring.color;
            let dim_color = Color::lerp(ring_color, Color::new(0, 0, 0), 0.5);

            // Precompute the 5 random disperse coordinates near the ring point.
            let mut disperse_coords: Vec<Coord> = Vec::with_capacity(5);
            for _ in 0..5 {
                let col = (ring_coord.column + rng.gen_range(-ring_gap, ring_gap)).clamp(1, width);
                let row = (ring_coord.row + rng.gen_range(-ring_gap, ring_gap)).clamp(1, height);
                disperse_coords.push(Coord::new(col, row));
            }

            let sym = infos[m.idx].0;
            let chars = terminal.get_characters_mut();
            let ch = &mut chars[m.idx];

            // Path to the assigned ring point.
            {
                let p = ch.motion.new_path("ring", 0.8, Some(easing::out_sine));
                p.new_waypoint("0", ring_coord);
            }
            // Rotation path: once around the ring, ending back at own point.
            {
                let p = ch.motion.new_path("rot", 0.2, None);
                for k in 1..=n {
                    let coord = rings[m.ring].coords[(m.point + k) % n];
                    p.new_waypoint(&k.to_string(), coord);
                }
            }
            // Disperse path.
            {
                let p = ch.motion.new_path("disperse", 0.14, None);
                for (k, coord) in disperse_coords.iter().enumerate() {
                    p.new_waypoint(&k.to_string(), *coord);
                }
            }
            // Sparkle scene used while dispersed (looping).
            {
                let scn = ch.animation.new_scene("sparkle", true);
                scn.add_frame(sym, 3, Some(ColorPair::fg_only(ring_color)));
                scn.add_frame(sym, 3, Some(ColorPair::fg_only(dim_color)));
            }

            ch.motion.activate_path("ring");
        }

        for &idx in &externals {
            let (sym, coord) = infos[idx];
            let start = start_color_for(coord);
            let fade = Gradient::new(&[start, gray], 8);
            let chars = terminal.get_characters_mut();
            let ch = &mut chars[idx];
            {
                let scn = ch.animation.new_scene("fade", false);
                for c in &fade.spectrum {
                    scn.add_frame(sym, 2, Some(ColorPair::fg_only(*c)));
                }
            }
            ch.animation.activate_scene("fade");
        }

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        // ---- Phase A: assemble the rings ------------------------------------
        let mut ticks = 0usize;
        while frames.len() < MAX_FRAMES && ticks < ASSEMBLE_TICK_CAP {
            let all_arrived = members
                .iter()
                .all(|m| terminal.get_characters()[m.idx].motion.movement_is_complete());
            let externals_settled = externals
                .iter()
                .all(|&i| terminal.get_characters()[i].animation.active_scene_is_complete());
            if all_arrived && externals_settled {
                break;
            }
            terminal.tick();
            frames.push(terminal.render_frame());
            ticks += 1;
        }

        // ---- Phase B: spin / disperse cycles ---------------------------------
        if !members.is_empty() {
            let mut spin_phase = |terminal: &mut Terminal, frames: &mut Vec<String>| {
                {
                    let chars = terminal.get_characters_mut();
                    for m in &members {
                        let sym = infos[m.idx].0;
                        let color = rings[m.ring].color;
                        let ch = &mut chars[m.idx];
                        ch.animation.deactivate_scene();
                        ch.animation
                            .set_appearance(sym, Some(ColorPair::fg_only(color)));
                        ch.motion.activate_path("rot");
                    }
                }
                for _ in 0..SPIN_TICKS {
                    if frames.len() >= MAX_FRAMES {
                        break;
                    }
                    {
                        let chars = terminal.get_characters_mut();
                        for m in &members {
                            if chars[m.idx].motion.movement_is_complete() {
                                chars[m.idx].motion.activate_path("rot");
                            }
                        }
                    }
                    terminal.tick();
                    frames.push(terminal.render_frame());
                }
            };

            let mut disperse_phase = |terminal: &mut Terminal, frames: &mut Vec<String>| {
                {
                    let chars = terminal.get_characters_mut();
                    for m in &members {
                        let ch = &mut chars[m.idx];
                        ch.animation.activate_scene("sparkle");
                        ch.motion.activate_path("disperse");
                    }
                }
                for _ in 0..DISPERSE_TICKS {
                    if frames.len() >= MAX_FRAMES {
                        break;
                    }
                    {
                        let chars = terminal.get_characters_mut();
                        for m in &members {
                            if chars[m.idx].motion.movement_is_complete() {
                                chars[m.idx].motion.activate_path("disperse");
                            }
                        }
                    }
                    terminal.tick();
                    frames.push(terminal.render_frame());
                }
            };

            for _ in 0..SPIN_DISPERSE_CYCLES {
                spin_phase(&mut terminal, &mut frames);
                disperse_phase(&mut terminal, &mut frames);
            }
            // Final spin before returning home.
            spin_phase(&mut terminal, &mut frames);
        }

        // ---- Phase C: return home with the final gradient --------------------
        {
            // Current color per character entering the final phase.
            let mut from_colors: Vec<Color> = infos
                .iter()
                .map(|&(_, coord)| start_color_for(coord))
                .collect();
            for m in &members {
                from_colors[m.idx] = rings[m.ring].color;
            }
            for &idx in &externals {
                from_colors[idx] = gray;
            }

            let chars = terminal.get_characters_mut();
            for (i, ch) in chars.iter_mut().enumerate() {
                let (sym, home) = infos[i];
                ch.animation.deactivate_scene();
                {
                    let p = ch.motion.new_path("home", 0.8, Some(easing::out_quad));
                    p.new_waypoint("0", home);
                }
                ch.motion.activate_path("home");

                let final_color = start_color_for(home);
                let grad = Gradient::new(&[from_colors[i], final_color], 8);
                {
                    let scn = ch.animation.new_scene("final", false);
                    for c in &grad.spectrum {
                        scn.add_frame(sym, 2, Some(ColorPair::fg_only(*c)));
                    }
                }
                ch.animation.activate_scene("final");
            }
        }

        while terminal.is_active() && frames.len() < MAX_FRAMES {
            terminal.tick();
            frames.push(terminal.render_frame());
        }
        frames.push(terminal.render_frame());

        frames
    }
}
