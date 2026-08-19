use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

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
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);

        let mut term = Terminal::from_input(input, width, height);
        let n = term.get_characters().len();
        if n == 0 {
            return vec![String::new()];
        }

        let rain = Gradient::new(
            vec![
                Color::from_hex("1a237e").unwrap_or(Color::rgb(26, 35, 126)),
                Color::from_hex("42a5f5").unwrap_or(Color::rgb(66, 165, 245)),
                Color::from_hex("e3f2fd").unwrap_or(Color::rgb(227, 242, 253)),
            ],
            8,
        )
        .colors();
        let bolt = Color::from_hex("fff59d").unwrap_or(Color::rgb(255, 245, 157));
        let flash = Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255));
        let settle = Color::from_hex("90caf9").unwrap_or(Color::rgb(144, 202, 249));

        let mut frames = Vec::new();
        let rain_frames = (height as i32 + 4).max(8) as usize;
        let bolt_every = ((n / 6).max(4)).min(12);

        for t in 0..rain_frames {
            for (i, ch) in term.get_characters_mut().iter_mut().enumerate() {
                let dest = ch.input_coord;
                let start_row = dest.row - (height as i32 + 2) - ((i as i32) % 3);
                let progress = (t as f64 / rain_frames as f64).min(1.0);
                let row = start_row + ((dest.row - start_row) as f64 * progress).round() as i32;
                ch.current_coord = Coord::new(dest.column, row);
                ch.is_visible = row >= 0;
                let col = rain[i % rain.len()];
                ch.colors = Some(ColorPair {
                    fg: Some(col),
                    bg: None,
                });
                ch.animation.set_appearance(ch.input_symbol, Some(col));
            }
            frames.push(render_ansi(&term));
        }

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        for wave in 0..3 {
            for (i, ch) in term.get_characters_mut().iter_mut().enumerate() {
                ch.current_coord = ch.input_coord;
                let struck = (i + wave * 7) % bolt_every == 0;
                let col = if struck { flash } else { rain[i % rain.len()] };
                ch.colors = Some(ColorPair {
                    fg: Some(col),
                    bg: if struck {
                        Some(Color::rgb(40, 40, 80))
                    } else {
                        None
                    },
                });
                ch.animation.set_appearance(ch.input_symbol, Some(col));
            }
            frames.push(render_ansi(&term));
            for (i, ch) in term.get_characters_mut().iter_mut().enumerate() {
                let struck = (i + wave * 7) % bolt_every == 0;
                let col = if struck { bolt } else { settle };
                ch.colors = Some(ColorPair {
                    fg: Some(col),
                    bg: None,
                });
                ch.animation.set_appearance(ch.input_symbol, Some(col));
            }
            frames.push(render_ansi(&term));
        }

        for (i, ch) in term.get_characters_mut().iter_mut().enumerate() {
            ch.current_coord = ch.input_coord;
            let col = rain[(i + rain.len() / 2) % rain.len()];
            ch.colors = Some(ColorPair {
                fg: Some(col),
                bg: None,
            });
            ch.animation.set_appearance(ch.input_symbol, Some(col));
        }
        frames.push(render_ansi(&term));
        frames
    }
}

fn sgr(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid = vec![vec![(' ', None::<Color>); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let fg = ch
                .animation
                .current_character_visual
                .colors
                .and_then(|p| p.fg)
                .or_else(|| ch.colors.and_then(|p| p.fg));
            grid[y as usize][x as usize] = (ch.animation.current_character_visual.symbol, fg);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, fg) in row {
            if let Some(c) = fg {
                out.push_str(&sgr(c));
                out.push(sym);
                out.push_str("\x1b[0m");
            } else {
                out.push(sym);
            }
        }
        out.push('\n');
    }
    out
}
