//! VHS tape glitch effect: rows of text glitch horizontally with tracking-line
//! colors, a glitch wave sweeps the text, everything degrades to color noise,
//! and finally the text restores row by row (port of effect_vhstape.py).

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

use std::collections::BTreeMap;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaveState {
    Pending(u32),
    Mid,
    End,
    Restoring,
    Done,
}

pub struct Vhstape;

impl Vhstape {
    pub fn new() -> Self {
        Self
    }
}

fn activate_scene(term: &mut Terminal, idx: usize, scene_id: &str) {
    term.get_characters_mut()[idx].animation.activate_scene(scene_id);
}

fn activate_path(term: &mut Terminal, idx: usize, path_id: &str) {
    term.get_characters_mut()[idx].motion.activate_path(path_id);
}

fn row_motion_done(term: &Terminal, row: &[usize]) -> bool {
    row.iter()
        .all(|&i| term.get_characters()[i].motion.movement_is_complete())
}

impl Effect for Vhstape {
    fn name(&self) -> &str {
        "vhstape"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut term = Terminal::new(input, TerminalConfig::default());
        let mut frames: Vec<String> = Vec::new();

        // --- palettes (mirroring the Python defaults) -----------------------
        let glitch_line_colors: Vec<Color> = ["ffffff", "ff0000", "00ff00", "0000ff", "ffffff"]
            .iter()
            .filter_map(|h| Color::from_hex(h))
            .collect();
        let noise_colors: Vec<Color> = ["1e1e1f", "3c3b3d", "6d6c70", "a2a1a6", "cb0003"]
            .iter()
            .filter_map(|h| Color::from_hex(h))
            .collect();
        let final_stops: Vec<Color> = ["ab48ff", "ea551f", "82f2ff"]
            .iter()
            .filter_map(|h| Color::from_hex(h))
            .collect();
        let final_gradient = Gradient::new(&final_stops, 12);
        let noise_symbols = ['#', '*', '.', ':'];
        let white = Color::new(255, 255, 255);

        // --- gather character info ------------------------------------------
        let infos: Vec<(usize, char, Coord)> = term
            .get_characters()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.input_symbol, c.input_coord))
            .collect();

        if infos.is_empty() {
            frames.push(term.render_frame());
            return frames;
        }

        let min_row = infos.iter().map(|(_, _, c)| c.row).min().unwrap_or(1);
        let max_row = infos.iter().map(|(_, _, c)| c.row).max().unwrap_or(1);

        // group by row, then order rows top -> bottom
        let mut by_row: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
        for (idx, _, coord) in &infos {
            by_row.entry(coord.row).or_default().push(*idx);
        }
        let rows: Vec<Vec<usize>> = by_row.values().rev().cloned().collect();

        // --- per-character scenes and paths ----------------------------------
        for &(idx, symbol, coord) in &infos {
            let fraction = if max_row == min_row {
                0.0
            } else {
                (max_row - coord.row) as f64 / (max_row - min_row) as f64
            };
            let final_color = final_gradient.get_color_at_fraction(fraction).unwrap_or(white);

            let ch = &mut term.get_characters_mut()[idx];

            // stable/base appearance
            let base = ch.animation.new_scene("base", false);
            base.add_frame(symbol, 1, Some(ColorPair::fg_only(final_color)));

            // glitch tracking-line flicker (white/red/green/blue/white)
            let glitch = ch.animation.new_scene("glitch", false);
            for _ in 0..2 {
                for color in &glitch_line_colors {
                    glitch.add_frame(symbol, 1, Some(ColorPair::fg_only(*color)));
                }
            }

            // degraded snow/noise (looping)
            let noise = ch.animation.new_scene("noise", true);
            for (i, color) in noise_colors.iter().enumerate() {
                let sym = noise_symbols[(idx + i) % noise_symbols.len()];
                noise.add_frame(sym, 2, Some(ColorPair::fg_only(*color)));
            }

            // final restored appearance
            let fin = ch.animation.new_scene("final", false);
            fin.add_frame(symbol, 1, Some(ColorPair::fg_only(final_color)));

            // motion paths
            let p = ch.motion.new_path("glitch_left", 2.0, None);
            p.new_waypoint("glitch_left", Coord::new(coord.column - 2, coord.row));
            let p = ch.motion.new_path("glitch_right", 2.0, None);
            p.new_waypoint("glitch_right", Coord::new(coord.column + 2, coord.row));
            let p = ch.motion.new_path("glitch_wave_mid", 2.0, None);
            p.new_waypoint("glitch_wave_mid", Coord::new(coord.column + 8, coord.row));
            let p = ch.motion.new_path("glitch_wave_end", 2.0, None);
            p.new_waypoint("glitch_wave_end", Coord::new(coord.column + 14, coord.row));
            let p = ch.motion.new_path("restore", 2.0, None);
            p.new_waypoint("restore", coord);

            ch.animation.activate_scene("base");
            ch.is_visible = true;
        }

        frames.push(term.render_frame());

        let mut rng = Lcg::new(0x5644_5354_4150_4531);

        // --- phase 1: intermittent row glitches -------------------------------
        let mut active_glitches: Vec<(usize, u32)> = Vec::new();
        for tick in 0..90u32 {
            if tick % 12 == 0 && !rows.is_empty() {
                let count = 1 + (rng.next_u32() % 2) as usize;
                for _ in 0..count {
                    let r = (rng.next_u32() as usize) % rows.len();
                    if active_glitches.iter().any(|(rr, _)| *rr == r) {
                        continue;
                    }
                    let path_id = if rng.next_u32() % 2 == 0 {
                        "glitch_left"
                    } else {
                        "glitch_right"
                    };
                    for &ci in &rows[r] {
                        activate_scene(&mut term, ci, "glitch");
                        activate_path(&mut term, ci, path_id);
                    }
                    active_glitches.push((r, 6));
                }
            }
            let mut restored: Vec<usize> = Vec::new();
            for entry in &mut active_glitches {
                if entry.1 > 0 {
                    entry.1 -= 1;
                }
                if entry.1 == 0 {
                    restored.push(entry.0);
                }
            }
            active_glitches.retain(|e| e.1 > 0);
            for r in restored {
                for &ci in &rows[r] {
                    activate_path(&mut term, ci, "restore");
                    activate_scene(&mut term, ci, "base");
                }
            }
            term.tick();
            frames.push(term.render_frame());
        }
        // force-restore anything still glitching
        for (r, _) in active_glitches.drain(..) {
            for &ci in &rows[r] {
                activate_path(&mut term, ci, "restore");
                activate_scene(&mut term, ci, "base");
            }
        }
        for _ in 0..6 {
            term.tick();
            frames.push(term.render_frame());
        }

        // --- phase 2: glitch wave sweeping top -> bottom ----------------------
        let mut states: Vec<WaveState> = rows
            .iter()
            .enumerate()
            .map(|(i, _)| WaveState::Pending(i as u32 * 3))
            .collect();
        let mut safety = 0u32;
        while states.iter().any(|s| *s != WaveState::Done) && safety < 2000 {
            for r in 0..rows.len() {
                match states[r] {
                    WaveState::Pending(0) => {
                        for &ci in &rows[r] {
                            activate_scene(&mut term, ci, "glitch");
                            activate_path(&mut term, ci, "glitch_wave_mid");
                        }
                        states[r] = WaveState::Mid;
                    }
                    WaveState::Pending(t) => {
                        states[r] = WaveState::Pending(t - 1);
                    }
                    WaveState::Mid => {
                        if row_motion_done(&term, &rows[r]) {
                            for &ci in &rows[r] {
                                activate_path(&mut term, ci, "glitch_wave_end");
                            }
                            states[r] = WaveState::End;
                        }
                    }
                    WaveState::End => {
                        if row_motion_done(&term, &rows[r]) {
                            for &ci in &rows[r] {
                                activate_path(&mut term, ci, "restore");
                            }
                            states[r] = WaveState::Restoring;
                        }
                    }
                    WaveState::Restoring => {
                        if row_motion_done(&term, &rows[r]) {
                            for &ci in &rows[r] {
                                activate_scene(&mut term, ci, "base");
                            }
                            states[r] = WaveState::Done;
                        }
                    }
                    WaveState::Done => {}
                }
            }
            term.tick();
            frames.push(term.render_frame());
            safety += 1;
        }

        // --- phase 3: full-screen tape noise ----------------------------------
        let all_indices: Vec<usize> = infos.iter().map(|(i, _, _)| *i).collect();
        for &ci in &all_indices {
            activate_scene(&mut term, ci, "noise");
        }
        for _ in 0..30 {
            term.tick();
            frames.push(term.render_frame());
        }

        // --- phase 4: staggered restore, top -> bottom ------------------------
        let mut started = vec![false; rows.len()];
        let total_ticks = rows.len() * 2 + 6;
        for tick in 0..total_ticks {
            for (r, row) in rows.iter().enumerate() {
                if !started[r] && tick >= r * 2 {
                    started[r] = true;
                    for &ci in row {
                        activate_scene(&mut term, ci, "final");
                    }
                }
            }
            term.tick();
            frames.push(term.render_frame());
        }

        frames
    }
}
