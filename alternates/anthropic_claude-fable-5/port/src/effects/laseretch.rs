//! Laser etch effect (port of terminaltexteffects/effects/effect_laseretch.py).
//!
//! A laser emitter in the top-right corner of the canvas fires a beam at each
//! character position in turn (top row to bottom row, left to right). As the
//! beam strikes, the character is etched: it sparks white-hot, cools through
//! an ember gradient, and settles on its final color taken from a vertical
//! final gradient across the text.

use std::collections::VecDeque;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::geometry::{find_length_of_line, lerp_coord, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// The `laseretch` effect.
pub struct Laseretch;

impl Laseretch {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Laseretch {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Laseretch {
    fn name(&self) -> &str {
        "laseretch"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;

        // --- palette (mirrors the Python defaults) -------------------------
        let spark_color = Color::from_hex("ffffff").expect("valid hex");
        let cool_a = Color::from_hex("ffe680").expect("valid hex");
        let cool_b = Color::from_hex("ff9100").expect("valid hex");
        let laser_gradient = Gradient::new(
            &[
                Color::from_hex("ffffff").expect("valid hex"),
                Color::from_hex("ff0000").expect("valid hex"),
            ],
            12,
        );
        let final_stops = [
            Color::from_hex("8A008A").expect("valid hex"),
            Color::from_hex("00D1FF").expect("valid hex"),
            Color::from_hex("ffffff").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&final_stops, 10);

        // --- final gradient mapping across the text's row extent -----------
        let (min_row, max_row) = {
            let mut min_row = i32::MAX;
            let mut max_row = i32::MIN;
            for character in terminal.get_characters() {
                min_row = min_row.min(character.input_coord.row);
                max_row = max_row.max(character.input_coord.row);
            }
            if min_row > max_row {
                (1, 1)
            } else {
                (min_row, max_row)
            }
        };

        // --- etch order: top row to bottom row, left to right --------------
        let mut order: Vec<(u32, Coord)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.character_id, c.input_coord))
            .collect();
        order.sort_by(|a, b| b.1.row.cmp(&a.1.row).then(a.1.column.cmp(&b.1.column)));

        // --- build the spark -> cool -> final scene for every character ----
        for (id, coord) in &order {
            let t = if max_row == min_row {
                0.0
            } else {
                (max_row - coord.row) as f64 / (max_row - min_row) as f64
            };
            let final_color = final_gradient
                .get_color_at_fraction(t)
                .unwrap_or(final_stops[2]);
            let cool = Gradient::new(&[spark_color, cool_a, cool_b, final_color], 3);
            let character = &mut terminal.get_characters_mut()[*id as usize];
            let symbol = character.input_symbol;
            let scene = character.animation.new_scene("etch", false);
            for color in &cool.spectrum {
                scene.add_frame(symbol, 2, Some(ColorPair::fg_only(*color)));
            }
        }

        // --- pool of characters used to draw the beam ----------------------
        let emitter = Coord::new(width, height);
        let beam_len = (width + height) as usize + 2;
        let mut beam_ids: Vec<u32> = Vec::with_capacity(beam_len);
        for _ in 0..beam_len {
            beam_ids.push(terminal.add_character(' ', emitter));
        }

        let mut pending: VecDeque<(u32, Coord)> = order.into_iter().collect();

        let mut frames = Vec::new();
        frames.push(terminal.render_frame());
        let max_frames = 20_000usize;

        loop {
            // Etch the next character, if any remain.
            let target = pending.pop_front();
            if let Some((id, _coord)) = target {
                let character = &mut terminal.get_characters_mut()[id as usize];
                character.is_visible = true;
                character.animation.activate_scene("etch");
            }

            // Hide all beam segments, then redraw the beam toward the target.
            for &bid in &beam_ids {
                terminal.get_characters_mut()[bid as usize].is_visible = false;
            }
            if let Some((_id, coord)) = target {
                let distance = find_length_of_line(emitter, coord);
                let steps = (distance.round() as i32).max(1);
                let dx = coord.column - emitter.column;
                let dy = coord.row - emitter.row;
                let symbol = if dx == 0 {
                    '|'
                } else if dy == 0 {
                    '_'
                } else if (dx as i64) * (dy as i64) > 0 {
                    '/'
                } else {
                    '\\'
                };
                for i in 0..steps {
                    let frac = i as f64 / steps as f64;
                    let point = lerp_coord(emitter, coord, frac);
                    if point == coord {
                        // Never cover the cell currently being etched.
                        continue;
                    }
                    let color = laser_gradient
                        .get_color_at_fraction(frac)
                        .unwrap_or(spark_color);
                    let idx = (i as usize).min(beam_ids.len() - 1);
                    let beam = &mut terminal.get_characters_mut()[beam_ids[idx] as usize];
                    beam.motion.current_coord = point;
                    let glyph = if i == 0 { '▼' } else { symbol };
                    beam.animation
                        .set_appearance(glyph, Some(ColorPair::fg_only(color)));
                    beam.is_visible = true;
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());

            if frames.len() >= max_frames {
                break;
            }
            if pending.is_empty() && target.is_none() && !terminal.is_active() {
                break;
            }
        }

        frames
    }
}
