//! Print effect: lines are "printed" one at a time by a typing head that
//! types each character at the bottom text row, then performs a carriage
//! return while previously printed rows scroll upward (a Rust port of
//! terminaltexteffects/effects/effect_print.py).

use std::collections::{BTreeMap, VecDeque};

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const MAX_FRAMES: usize = 5000;

/// The `print` effect.
pub struct Print;

impl Print {
    pub fn new() -> Self {
        Print
    }
}

impl Effect for Print {
    fn name(&self) -> &str {
        "print"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;

        // Final gradient (upstream defaults: 02b8bd -> c1f0e3 -> 00ffa0, 12 steps,
        // diagonal direction).
        let stops = [
            Color::from_hex("02b8bd").expect("valid hex"),
            Color::from_hex("c1f0e3").expect("valid hex"),
            Color::from_hex("00ffa0").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&stops, 12);
        let head_color = *final_gradient
            .spectrum
            .last()
            .expect("gradient has at least one color");

        // Group characters into rows (BTreeMap keys ascend: first key = bottom row).
        let mut rows_map: BTreeMap<i32, Vec<(i32, u32)>> = BTreeMap::new();
        for ch in terminal.get_characters() {
            rows_map
                .entry(ch.input_coord.row)
                .or_default()
                .push((ch.input_coord.column, ch.character_id));
        }
        if rows_map.is_empty() {
            return vec![terminal.render_frame()];
        }
        let bottom_row = *rows_map.keys().next().expect("non-empty rows");

        // Rows ordered top-to-bottom, each row ordered left-to-right.
        let rows: Vec<Vec<u32>> = rows_map
            .iter()
            .rev()
            .map(|(_, cells)| {
                let mut cells = cells.clone();
                cells.sort_by_key(|(column, _)| *column);
                cells.into_iter().map(|(_, id)| id).collect()
            })
            .collect();

        // Per-character final color, diagonal gradient direction.
        let mut final_colors: BTreeMap<u32, Color> = BTreeMap::new();
        for ch in terminal.get_characters() {
            let tx = if width > 1 {
                (ch.input_coord.column - 1) as f64 / (width - 1) as f64
            } else {
                0.0
            };
            let ty = if height > 1 {
                (height - ch.input_coord.row) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let t = (tx + ty) / 2.0;
            final_colors.insert(
                ch.character_id,
                final_gradient.get_color_at_fraction(t).unwrap_or(head_color),
            );
        }

        // The typing head (hidden except during carriage returns).
        let head_id = terminal.add_character('█', Coord::new(1, bottom_row));
        {
            let chars = terminal.get_characters_mut();
            let head = &mut chars[head_id as usize];
            head.animation
                .set_appearance('█', Some(ColorPair::fg_only(head_color)));
        }

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let mut pending_rows = rows.into_iter();
        let mut current_row: VecDeque<u32> = VecDeque::new();
        let mut typed_ids: Vec<u32> = Vec::new();
        let mut done_typing = false;
        let mut return_count: usize = 0;

        while frames.len() < MAX_FRAMES {
            let head_moving = !terminal.get_characters()[head_id as usize]
                .motion
                .movement_is_complete();

            if !head_moving && !done_typing {
                if let Some(next_id) = current_row.pop_front() {
                    // Type the next character at the bottom text row.
                    terminal.set_character_visibility(head_id, false);
                    let final_color = final_colors[&next_id];
                    let typed_gradient = Gradient::new(&[head_color, final_color], 4);
                    let column;
                    {
                        let chars = terminal.get_characters_mut();
                        let ch = &mut chars[next_id as usize];
                        column = ch.input_coord.column;
                        ch.motion.current_coord = Coord::new(column, bottom_row);
                        let input_symbol = ch.input_symbol;
                        let symbols = ['█', '▓', '▒', '░', input_symbol];
                        let scene = ch.animation.new_scene("typed", false);
                        for (i, symbol) in symbols.iter().enumerate() {
                            let t = i as f64 / (symbols.len() - 1) as f64;
                            let color = typed_gradient
                                .get_color_at_fraction(t)
                                .unwrap_or(final_color);
                            scene.add_frame(*symbol, 3, Some(ColorPair::fg_only(color)));
                        }
                        ch.animation.activate_scene("typed");
                        ch.is_visible = true;
                    }
                    typed_ids.push(next_id);
                    // Park the (hidden) head just right of the typed character so the
                    // next carriage return starts from the end of the row.
                    {
                        let chars = terminal.get_characters_mut();
                        let head = &mut chars[head_id as usize];
                        head.motion.current_coord =
                            Coord::new((column + 1).min(width), bottom_row);
                    }
                } else if let Some(next_row) = pending_rows.next() {
                    // Feed the paper: every printed row scrolls up one line.
                    {
                        let chars = terminal.get_characters_mut();
                        for &id in &typed_ids {
                            let ch = &mut chars[id as usize];
                            ch.motion.current_coord = Coord::new(
                                ch.motion.current_coord.column,
                                ch.motion.current_coord.row + 1,
                            );
                        }
                    }
                    // Carriage return to the first character of the next row.
                    let target_column = next_row
                        .first()
                        .map(|&id| terminal.get_characters()[id as usize].input_coord.column)
                        .unwrap_or(1);
                    current_row = next_row.into_iter().collect();
                    return_count += 1;
                    let path_id = format!("carriage_return_{return_count}");
                    {
                        let chars = terminal.get_characters_mut();
                        let head = &mut chars[head_id as usize];
                        head.animation
                            .set_appearance('█', Some(ColorPair::fg_only(head_color)));
                        let path =
                            head.motion
                                .new_path(&path_id, 1.25, Some(easing::in_out_quad));
                        path.new_waypoint("target", Coord::new(target_column, bottom_row));
                        head.motion.activate_path(&path_id);
                        head.is_visible = true;
                    }
                } else {
                    done_typing = true;
                    terminal.set_character_visibility(head_id, false);
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());

            if done_typing && !terminal.is_active() {
                break;
            }
        }

        frames
    }
}
