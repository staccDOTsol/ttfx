//! Synthgrid: a neon grid expands over the canvas, text characters dissolve
//! into place inside the grid sections, then the grid collapses away.
//! Port of terminaltexteffects/effects/effect_synthgrid.py (simplified to the
//! engine facilities available in this crate).

use std::collections::HashMap;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const GRID_ROW_SYMBOL: char = '─';
const GRID_COLUMN_SYMBOL: char = '│';
const TEXT_GENERATION_SYMBOLS: [char; 3] = ['░', '▒', '▓'];
const MAX_ACTIVE_BLOCKS: f64 = 0.1;
const MAX_FRAMES: usize = 3000;

/// Small deterministic PRNG (no external rng module exists in this crate).
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 ^ (self.0 >> 31)
    }

    fn gen_range(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }
}

fn shuffle<T>(v: &mut [T], rng: &mut Lcg) {
    if v.len() < 2 {
        return;
    }
    for i in (1..v.len()).rev() {
        let j = rng.gen_range(i + 1);
        v.swap(i, j);
    }
}

/// Reorder a line's character ids so they reveal from the center outward,
/// mirroring the Python grid lines extending from their midpoint.
fn center_out(ids: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(ids.len());
    let mid = ids.len() / 2;
    let mut left: isize = mid as isize - 1;
    let mut right = mid;
    while left >= 0 || right < ids.len() {
        if right < ids.len() {
            out.push(ids[right]);
            right += 1;
        }
        if left >= 0 {
            out.push(ids[left as usize]);
            left -= 1;
        }
    }
    out
}

pub struct Synthgrid;

impl Synthgrid {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Synthgrid {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Synthgrid {
    fn name(&self) -> &str {
        "synthgrid"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let w = terminal.canvas.width;
        let h = terminal.canvas.height;

        let mut rng = Lcg::new(0x5eed_5919_71d5 ^ (input.len() as u64) ^ ((w as u64) << 24) ^ (h as u64));

        // Gradients (defaults inspired by the Python effect config).
        let grid_gradient = Gradient::new(
            &[
                Color::from_hex("CC00CC").expect("valid hex"),
                Color::from_hex("ffffff").expect("valid hex"),
            ],
            12,
        );
        let text_gradient = Gradient::new(
            &[
                Color::from_hex("8A008A").expect("valid hex"),
                Color::from_hex("00D1FF").expect("valid hex"),
                Color::from_hex("FFFFFF").expect("valid hex"),
            ],
            12,
        );

        // Snapshot the input text characters before adding grid characters.
        let text_chars: Vec<(u32, Coord)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.character_id, c.input_coord))
            .collect();

        // ------------------------------------------------------------------
        // Build the grid lines (border + interior lines spaced by the gaps).
        // ------------------------------------------------------------------
        let col_gap = ((w as f64 / 8.0).round() as i32).max(2);
        let row_gap = ((h as f64 / 4.0).round() as i32).max(2);

        let mut grid_rows: Vec<i32> = Vec::new();
        let mut r = 1;
        while r <= h {
            grid_rows.push(r);
            r += row_gap;
        }
        if grid_rows.last() != Some(&h) {
            grid_rows.push(h);
        }

        let mut grid_cols: Vec<i32> = Vec::new();
        let mut c = 1;
        while c <= w {
            grid_cols.push(c);
            c += col_gap;
        }
        if grid_cols.last() != Some(&w) {
            grid_cols.push(w);
        }

        let first_grid_id = text_chars.len() as u32;
        let mut lines: Vec<Vec<u32>> = Vec::new();
        let mut all_grid_ids: Vec<u32> = Vec::new();

        for &row in &grid_rows {
            let mut line_ids = Vec::with_capacity(w as usize);
            for col in 1..=w {
                let id = terminal.add_character(GRID_ROW_SYMBOL, Coord::new(col, row));
                line_ids.push(id);
                all_grid_ids.push(id);
            }
            lines.push(center_out(&line_ids));
        }
        for &col in &grid_cols {
            let mut line_ids = Vec::with_capacity(h as usize);
            for row in 1..=h {
                let id = terminal.add_character(GRID_COLUMN_SYMBOL, Coord::new(col, row));
                line_ids.push(id);
                all_grid_ids.push(id);
            }
            lines.push(center_out(&line_ids));
        }

        // Color the grid characters with a diagonal sweep of the grid gradient,
        // and build the dissolve scenes for the text characters.
        for ch in terminal.get_characters_mut() {
            if ch.character_id >= first_grid_id {
                let fraction =
                    (ch.input_coord.column + ch.input_coord.row) as f64 / (w + h).max(1) as f64;
                let color = grid_gradient
                    .get_color_at_fraction(fraction)
                    .unwrap_or(Color::new(255, 255, 255));
                ch.animation
                    .set_appearance(ch.input_symbol, Some(ColorPair::fg_only(color)));
            } else {
                let final_fraction = if h > 1 {
                    (ch.input_coord.row - 1) as f64 / (h - 1) as f64
                } else {
                    0.0
                };
                let final_color = text_gradient
                    .get_color_at_fraction(final_fraction)
                    .unwrap_or(Color::new(255, 255, 255));
                let input_symbol = ch.input_symbol;
                let scene = ch.animation.new_scene("dissolve", false);
                for i in 0..12 {
                    let symbol =
                        TEXT_GENERATION_SYMBOLS[rng.gen_range(TEXT_GENERATION_SYMBOLS.len())];
                    let color = text_gradient
                        .get_color_at_fraction(i as f64 / 11.0)
                        .unwrap_or(Color::new(255, 255, 255));
                    scene.add_frame(symbol, 2, Some(ColorPair::fg_only(color)));
                }
                scene.add_frame(input_symbol, 1, Some(ColorPair::fg_only(final_color)));
            }
        }

        let mut frames: Vec<String> = Vec::new();

        // ------------------------------------------------------------------
        // Phase 1: grid lines extend from their centers outward.
        // ------------------------------------------------------------------
        let reveal_ticks = 20usize;
        let mut line_progress: Vec<usize> = vec![0; lines.len()];
        for _ in 0..reveal_ticks {
            for (li, line) in lines.iter().enumerate() {
                let chunk = line.len().div_ceil(reveal_ticks).max(1);
                let end = (line_progress[li] + chunk).min(line.len());
                for &id in &line[line_progress[li]..end] {
                    terminal.set_character_visibility(id, true);
                }
                line_progress[li] = end;
            }
            terminal.tick();
            frames.push(terminal.render_frame());
        }
        for line in &lines {
            for &id in line {
                terminal.set_character_visibility(id, true);
            }
        }

        // ------------------------------------------------------------------
        // Phase 2: text characters dissolve in, grid-section by grid-section,
        // with a limited fraction of sections active at once.
        // ------------------------------------------------------------------
        let mut sections: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for &(id, coord) in &text_chars {
            let key = ((coord.column - 1) / col_gap, (coord.row - 1) / row_gap);
            sections.entry(key).or_default().push(id);
        }
        let mut section_queue: Vec<Vec<u32>> = sections.into_values().collect();
        shuffle(&mut section_queue, &mut rng);

        let max_active = ((section_queue.len() as f64 * MAX_ACTIVE_BLOCKS).round() as usize).max(1);
        let mut active_sections: Vec<Vec<u32>> = Vec::new();

        while (!section_queue.is_empty() || !active_sections.is_empty())
            && frames.len() < MAX_FRAMES
        {
            while active_sections.len() < max_active {
                let Some(section) = section_queue.pop() else {
                    break;
                };
                for ch in terminal.get_characters_mut() {
                    if section.contains(&ch.character_id) {
                        ch.is_visible = true;
                        ch.animation.activate_scene("dissolve");
                    }
                }
                active_sections.push(section);
            }

            terminal.tick();
            frames.push(terminal.render_frame());

            let chars = terminal.get_characters();
            active_sections.retain(|section| {
                section.iter().any(|&id| {
                    chars
                        .iter()
                        .find(|c| c.character_id == id)
                        .map(|c| !c.animation.active_scene_is_complete())
                        .unwrap_or(false)
                })
            });
        }

        // ------------------------------------------------------------------
        // Phase 3: the grid collapses away, leaving the styled text.
        // ------------------------------------------------------------------
        shuffle(&mut all_grid_ids, &mut rng);
        let collapse_chunk = (all_grid_ids.len() / 15).max(1);
        while !all_grid_ids.is_empty() && frames.len() < MAX_FRAMES {
            for _ in 0..collapse_chunk {
                if let Some(id) = all_grid_ids.pop() {
                    terminal.set_character_visibility(id, false);
                }
            }
            terminal.tick();
            frames.push(terminal.render_frame());
        }

        // A few settle frames with the final colored text.
        for _ in 0..3 {
            if frames.len() >= MAX_FRAMES {
                break;
            }
            terminal.tick();
            frames.push(terminal.render_frame());
        }

        frames
    }
}
