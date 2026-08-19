use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Beams;

impl Beams {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Beams {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Beams {
    fn name(&self) -> &str {
        "beams"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let width = input
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = input.lines().count().max(1);
        let mut term = Terminal::from_input(input, width, height);

        let beam_stops = vec![
            Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
            Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
            Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let final_stops = beam_stops.clone();
        let beam_grad = Gradient::new(beam_stops, 12);
        let beam_colors = beam_grad.colors();
        let final_grad = Gradient::new(final_stops, (width + height).max(8));
        let final_colors = final_grad.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        let coords: Vec<(CharacterId, Coord, char)> = term
            .get_characters()
            .iter()
            .map(|c| (c.id, c.input_coord, c.input_symbol))
            .collect();

        for (id, coord, symbol) in &coords {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let idx = ((coord.column + coord.row).max(0) as usize) % final_colors.len().max(1);
                let fg = final_colors.get(idx).copied();
                let scn = ch.animation.new_scene("final");
                scn.add_frame(
                    *symbol,
                    1,
                    Some(ColorPair { fg, bg: None }),
                );
                let beam_scn = ch.animation.new_scene("beam");
                for (i, col) in beam_colors.iter().enumerate() {
                    beam_scn.add_frame(
                        if i % 2 == 0 { '▁' } else { '▂' },
                        2,
                        Some(ColorPair {
                            fg: Some(*col),
                            bg: None,
                        }),
                    );
                }
                beam_scn.add_frame(
                    *symbol,
                    4,
                    Some(ColorPair { fg, bg: None }),
                );
            }
        }

        let mut frames: Vec<String> = Vec::new();
        let max_col = coords.iter().map(|(_, c, _)| c.column).max().unwrap_or(0);
        let max_row = coords.iter().map(|(_, c, _)| c.row).max().unwrap_or(0);

        let mut revealed: Vec<bool> = vec![false; ids.len()];

        for sweep in 0..=(max_col + max_row + 8) {
            for (i, (id, coord, _)) in coords.iter().enumerate() {
                let hit = coord.column == sweep || coord.row == sweep - max_col / 2;
                if hit {
                    revealed[i] = true;
                    term.set_character_visibility(*id, true);
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                        ch.animation.activate_scene("beam");
                    }
                } else if revealed[i] {
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                        ch.animation.activate_scene("final");
                    }
                }
            }
            term.step_all();
            frames.push(render_ansi(&term));
        }

        for _ in 0..8 {
            for id in &ids {
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    ch.animation.activate_scene("final");
                }
                term.set_character_visibility(*id, true);
            }
            term.step_all();
            frames.push(render_ansi(&term));
        }

        if frames.is_empty() {
            frames.push(String::from("\x1b[38;2;138;0;138m \x1b[0m\n"));
        }
        frames
    }
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<(char, Option<Color>)>> = vec![vec![(' ', None); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 {
            let ux = x as usize;
            let uy = y as usize;
            if ux < w && uy < h {
                let sym = ch.animation.current_character_visual.symbol;
                let fg = ch
                    .animation
                    .current_character_visual
                    .colors
                    .and_then(|p| p.fg)
                    .or_else(|| ch.colors.and_then(|p| p.fg));
                grid[uy][ux] = (sym, fg);
            }
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, fg) in row {
            if let Some(c) = fg {
                out.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, sym));
            } else if sym != ' ' {
                out.push_str("\x1b[38;2;0;209;255m");
                out.push(sym);
                out.push_str("\x1b[0m");
            } else {
                out.push(' ');
            }
        }
        out.push('\n');
    }
    out
}
