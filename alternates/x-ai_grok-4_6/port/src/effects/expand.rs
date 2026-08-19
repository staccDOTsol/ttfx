use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Expand;

impl Expand {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Expand {
    fn name(&self) -> &str {
        "expand"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let center = Coord {
            column: (width as i32) / 2,
            row: (height as i32) / 2,
        };

        let gradient = Gradient::new(
            vec![
                Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
                Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
                Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
            ],
            12,
        );
        let palette = gradient.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let dist = ch.input_coord.distance(center);
                let idx = if palette.is_empty() {
                    0
                } else {
                    ((dist * 0.35) as usize) % palette.len()
                };
                let color = palette.get(idx).copied().unwrap_or(Color::rgb(0, 209, 255));
                let scn = ch.animation.new_scene("expand");
                scn.add_frame(
                    ch.input_symbol,
                    1,
                    Some(ColorPair {
                        fg: Some(color),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("expand");
                ch.motion.current_coord = center;
                ch.current_coord = center;
                let path = ch.motion.new_path("home");
                path.speed = 0.35;
                path.easing = Easing::OutCubic;
                path.new_waypoint("dest", ch.input_coord);
                ch.motion.activate_path("home");
            }
            term.set_character_visibility(*id, true);
        }

        let mut out = Vec::new();
        for _ in 0..90 {
            term.step_all();
            out.push(render_ansi(&term, width, height));
            let settled = term.get_characters().iter().all(|c| {
                c.current_coord.column == c.input_coord.column
                    && c.current_coord.row == c.input_coord.row
            });
            if settled && out.len() > 8 {
                break;
            }
        }
        if out.is_empty() {
            out.push(render_ansi(&term, width, height));
        }
        out
    }
}

fn render_ansi(term: &Terminal, width: usize, height: usize) -> String {
    let mut grid: Vec<Vec<(char, Option<ColorPair>)>> =
        vec![vec![(' ', None); width]; height];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
            let vis = &ch.animation.current_character_visual;
            grid[y as usize][x as usize] = (vis.symbol, vis.colors);
        }
    }
    let mut s = String::new();
    for row in grid {
        for (sym, colors) in row {
            if let Some(pair) = colors {
                if let Some(fg) = pair.fg {
                    s.push_str(&format!("\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b));
                }
                if let Some(bg) = pair.bg {
                    s.push_str(&format!("\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b));
                }
                s.push(sym);
                s.push_str("\x1b[0m");
            } else if sym != ' ' {
                s.push_str("\x1b[38;2;0;209;255m");
                s.push(sym);
                s.push_str("\x1b[0m");
            } else {
                s.push(' ');
            }
        }
        s.push('\n');
    }
    s
}
