use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Pour;

impl Pour {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Pour {
    fn name(&self) -> &str {
        "pour"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let gradient = Gradient::new(
            vec![
                Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
                Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
                Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
            ],
            8,
        );
        let palette = gradient.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        let mut order: Vec<(i32, CharacterId, Coord, char)> = term
            .get_characters()
            .iter()
            .map(|c| (c.input_coord.column + c.input_coord.row * 3, c.id, c.input_coord, c.input_symbol))
            .collect();
        order.sort_by_key(|(k, _, _, _)| *k);

        for (_k, id, input_coord, symbol) in &order {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let start = Coord::new(input_coord.column, -1);
                ch.motion.current_coord = start;
                ch.current_coord = start;
                let path = ch.motion.new_path("pour");
                path.speed = 0.18 + (input_coord.row as f64 * 0.01).min(0.25);
                path.easing = Easing::OutQuad;
                path.new_waypoint("home", *input_coord);
                let gi = ((input_coord.column + input_coord.row) as usize) % palette.len().max(1);
                let color = palette.get(gi).copied().unwrap_or(Color::rgb(0, 209, 255));
                let scn = ch.animation.new_scene("pour");
                scn.add_frame(*symbol, 1, Some(ColorPair { fg: Some(color), bg: None }));
                ch.animation.activate_scene("pour");
                ch.animation.set_appearance(*symbol, Some(color));
            }
        }

        let mut frames = Vec::new();
        let mut activated = 0usize;
        let pour_gap = 2usize;
        let max_frames = (height + width) * 8 + 40;

        for tick in 0..max_frames {
            if tick % pour_gap == 0 && activated < order.len() {
                let id = order[activated].1;
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == id) {
                    ch.motion.activate_path("pour");
                }
                term.set_character_visibility(id, true);
                activated += 1;
            }
            term.step_all();
            frames.push(render_ansi(&term));
            let all_home = term.get_characters().iter().all(|c| {
                !c.is_visible
                    || (c.current_coord.column == c.input_coord.column
                        && c.current_coord.row == c.input_coord.row)
            });
            if activated >= order.len() && all_home && tick > 4 {
                frames.push(render_ansi(&term));
                break;
            }
        }
        if frames.is_empty() {
            frames.push(render_ansi(&term));
        }
        let _ = ids;
        frames
    }
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<(char, Option<ColorPair>)>> = vec![vec![(' ', None); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let pair = ch
                .animation
                .current_character_visual
                .colors
                .or(ch.colors);
            grid[y as usize][x as usize] = (ch.animation.current_character_visual.symbol, pair);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, colors) in row {
            if let Some(pair) = colors {
                if let Some(fg) = pair.fg {
                    out.push_str(&format!("\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b));
                }
                if let Some(bg) = pair.bg {
                    out.push_str(&format!("\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b));
                }
                out.push(sym);
                out.push_str("\x1b[0m");
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
