//! Overflow effect: rows of the input text pour rapidly through the canvas
//! like a scrolling buffer overflow (tinted with an overflow gradient) before
//! the real rows finally settle into place colored with the final gradient.
//!
//! Port of terminaltexteffects/effects/effect_overflow.py.

use std::collections::{BTreeMap, HashMap};

use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Small deterministic xorshift PRNG (this crate has no rng module).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9e37_79b9_7f4a_7c15 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Inclusive range, like Python's random.randint.
    fn randint(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u64;
        lo + (self.next_u64() % span) as i64
    }

    /// Fisher-Yates shuffle, like Python's random.shuffle.
    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            v.swap(i, j);
        }
    }
}

/// One horizontal row of characters moving through the canvas.
struct Row {
    characters: Vec<CharacterId>,
    is_final: bool,
}

fn hex(s: &str) -> Color {
    Color::from_hex(s).expect("valid hex literal")
}

/// Python Row.move_up(): shift every character in the row up one cell.
fn move_up(terminal: &mut Terminal, row: &Row) {
    for &id in &row.characters {
        let character = &mut terminal.get_characters_mut()[id as usize];
        let coord = character.motion.current_coord;
        character.motion.current_coord = Coord::new(coord.column, coord.row + 1);
    }
}

/// Python Row.setup(): park the row just below the canvas at its input columns.
fn setup_row(terminal: &mut Terminal, row: &Row) {
    for &id in &row.characters {
        let character = &mut terminal.get_characters_mut()[id as usize];
        let column = character.input_coord.column;
        character.motion.current_coord = Coord::new(column, 0);
    }
}

/// Python Row.set_color(): recolor every character in the row.
fn set_row_color(terminal: &mut Terminal, row: &Row, color: Color) {
    for &id in &row.characters {
        let character = &mut terminal.get_characters_mut()[id as usize];
        let symbol = character.input_symbol;
        character
            .animation
            .set_appearance(symbol, Some(ColorPair::fg_only(color)));
    }
}

fn row_top(terminal: &Terminal, row: &Row) -> i32 {
    terminal.get_characters()[row.characters[0] as usize]
        .motion
        .current_coord
        .row
}

pub struct Overflow;

impl Overflow {
    pub fn new() -> Self {
        Self
    }

    fn run(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let top = terminal.canvas.height;
        let mut rng = Rng::new(0x0f10_c0de_5eed_u64);

        // --- final gradient (defaults: 8A008A -> 00D1FF -> FFFFFF, vertical) ---
        let final_stops = [hex("8A008A"), hex("00D1FF"), hex("FFFFFF")];
        let final_gradient = Gradient::new(&final_stops, 12);
        let mut final_color_map: HashMap<CharacterId, Color> = HashMap::new();
        for character in terminal.get_characters() {
            let t = if top > 1 {
                (character.input_coord.row - 1) as f64 / (top - 1) as f64
            } else {
                0.0
            };
            let color = final_gradient
                .get_color_at_fraction(t)
                .unwrap_or(final_stops[2]);
            final_color_map.insert(character.character_id, color);
        }

        // --- group original characters into rows, top-to-bottom ---
        let mut rows_map: BTreeMap<i32, Vec<(i32, CharacterId)>> = BTreeMap::new();
        for character in terminal.get_characters() {
            rows_map
                .entry(character.input_coord.row)
                .or_default()
                .push((character.input_coord.column, character.character_id));
        }
        let input_rows: Vec<Vec<CharacterId>> = rows_map
            .iter()
            .rev()
            .map(|(_, cells)| {
                let mut cells = cells.clone();
                cells.sort_by_key(|(column, _)| *column);
                cells.into_iter().map(|(_, id)| id).collect()
            })
            .collect();

        // --- overflow cycles: shuffled copies of every row (overflow_cycles_range 2..=4) ---
        let mut pending_rows: Vec<Row> = Vec::new();
        if !input_rows.is_empty() {
            let cycles = rng.randint(2, 4);
            let mut shuffled_rows = input_rows.clone();
            for _ in 0..cycles {
                rng.shuffle(&mut shuffled_rows);
                for row in &shuffled_rows {
                    let mut copies: Vec<CharacterId> = Vec::with_capacity(row.len());
                    for &id in row {
                        let (symbol, coord) = {
                            let source = &terminal.get_characters()[id as usize];
                            (source.input_symbol, source.input_coord)
                        };
                        copies.push(terminal.add_character(symbol, coord));
                    }
                    pending_rows.push(Row {
                        characters: copies,
                        is_final: false,
                    });
                }
            }
        }

        // --- final rows in correct order, pre-colored with the final gradient ---
        for row in &input_rows {
            for &id in row {
                let color = final_color_map
                    .get(&id)
                    .copied()
                    .unwrap_or(final_stops[2]);
                let character = &mut terminal.get_characters_mut()[id as usize];
                let symbol = character.input_symbol;
                character
                    .animation
                    .set_appearance(symbol, Some(ColorPair::fg_only(color)));
            }
            pending_rows.push(Row {
                characters: row.clone(),
                is_final: true,
            });
        }

        // --- overflow gradient (defaults: f2ebc0 -> 8dbfb3 -> f2ebc0) ---
        let overflow_stops = [hex("f2ebc0"), hex("8dbfb3"), hex("f2ebc0")];
        let overflow_steps = ((top / (overflow_stops.len() as i32 - 1)).max(1)) as usize;
        let overflow_gradient = Gradient::new(&overflow_stops, overflow_steps);
        let spectrum_len = overflow_gradient.spectrum.len().max(1);

        // --- frame loop (mirrors OverflowIterator.__next__) ---
        const OVERFLOW_SPEED: i64 = 3;
        let mut delay: i64 = 0;
        let mut active_rows: Vec<Row> = Vec::new();
        let mut frames: Vec<String> = vec![terminal.render_frame()];
        let mut guard = 0usize;

        while !pending_rows.is_empty() && guard < 10_000 {
            guard += 1;
            if delay == 0 {
                let burst = rng.randint(1, OVERFLOW_SPEED);
                for _ in 0..burst {
                    if pending_rows.is_empty() {
                        break;
                    }
                    // shift the active rows up and re-tint the overflow rows
                    for row in &active_rows {
                        move_up(&mut terminal, row);
                        if !row.is_final {
                            let current_row = row_top(&terminal, row).max(0) as usize;
                            let idx = current_row.min(spectrum_len - 1);
                            let color = overflow_gradient.spectrum[idx];
                            set_row_color(&mut terminal, row, color);
                        }
                    }
                    // feed the next row in from below the canvas
                    let next_row = pending_rows.remove(0);
                    setup_row(&mut terminal, &next_row);
                    move_up(&mut terminal, &next_row);
                    if !next_row.is_final {
                        set_row_color(&mut terminal, &next_row, overflow_gradient.spectrum[0]);
                    }
                    for &id in &next_row.characters {
                        terminal.set_character_visibility(id, true);
                    }
                    active_rows.push(next_row);
                }
                delay = rng.randint(0, 3);
            } else {
                delay -= 1;
            }
            // drop rows that have scrolled past the top of the canvas
            active_rows.retain(|row| row_top(&terminal, row) <= top);
            frames.push(terminal.render_frame());
        }

        frames
    }
}

impl Effect for Overflow {
    fn name(&self) -> &str {
        "overflow"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        self.run(input)
    }
}
