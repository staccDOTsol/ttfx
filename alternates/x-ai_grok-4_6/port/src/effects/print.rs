use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Print;

impl Print {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Print {
    fn name(&self) -> &str {
        "print"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = if input.is_empty() {
            vec![""]
        } else {
            input.lines().collect()
        };
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);

        let mut term = Terminal::from_input(input, width, height);
        let gradient = Gradient::new(
            vec![
                Color::from_hex("00c3ff").unwrap_or(Color::rgb(0, 195, 255)),
                Color::from_hex("ffff1c").unwrap_or(Color::rgb(255, 255, 28)),
                Color::from_hex("ff006e").unwrap_or(Color::rgb(255, 0, 110)),
            ],
            12,
        );
        let palette = gradient.colors();
        let n_pal = palette.len().max(1);

        let mut by_row: Vec<Vec<CharacterId>> = vec![Vec::new(); height];
        for ch in term.get_characters() {
            let r = ch.input_coord.row.max(0) as usize;
            if r < by_row.len() {
                by_row[r].push(ch.id);
            }
        }
        for row in &mut by_row {
            row.sort_by_key(|id| {
                term.get_characters()
                    .iter()
                    .find(|c| c.id == *id)
                    .map(|c| c.input_coord.column)
                    .unwrap_or(0)
            });
        }

        for ch in term.get_characters_mut() {
            let idx = ((ch.input_coord.column + ch.input_coord.row).unsigned_abs() as usize) % n_pal;
            let color = palette[idx];
            let pair = ColorPair {
                fg: Some(color),
                bg: None,
            };
            {
                let scn = ch.animation.new_scene("print");
                let ticks = [1u32, 2, 1, 3, 1];
                let glyphs = ['░', '▒', '▓', ch.input_symbol, ch.input_symbol];
                for (g, d) in glyphs.into_iter().zip(ticks) {
                    scn.add_frame(g, d, Some(pair));
                }
            }
            {
                let scn = ch.animation.new_scene("final");
                scn.add_frame(ch.input_symbol, 8, Some(pair));
            }
            let dest = ch.input_coord;
            let path = ch.motion.new_path("home");
            path.speed = 0.85;
            path.easing = Easing::OutQuad;
            path.new_waypoint("in", dest);
        }

        let head_id = term.add_character('█', Coord::new(0, 0));
        if let Some(head) = term.get_characters_mut().iter_mut().find(|c| c.id == head_id) {
            let pair = ColorPair {
                fg: Some(Color::rgb(255, 255, 255)),
                bg: None,
            };
            let scn = head.animation.new_scene("head");
            scn.is_looping = true;
            scn.add_frame('█', 2, Some(pair));
            scn.add_frame('▓', 2, Some(pair));
            head.animation.activate_scene("head");
        }
        term.set_character_visibility(head_id, true);

        let mut frames = Vec::new();
        let mut typed: Vec<CharacterId> = Vec::new();

        for (row_i, row_ids) in by_row.iter().enumerate() {
            if row_ids.is_empty() {
                continue;
            }
            if let Some(first) = term.get_characters().iter().find(|c| c.id == row_ids[0]) {
                let start = Coord::new(first.input_coord.column, row_i as i32);
                if let Some(head) = term.get_characters_mut().iter_mut().find(|c| c.id == head_id) {
                    head.motion.current_coord = start;
                    head.current_coord = start;
                    let p = head.motion.new_path(format!("cr{row_i}"));
                    p.speed = 1.2;
                    p.easing = Easing::InOutSine;
                    p.new_waypoint("col", start);
                    head.motion.activate_path(&format!("cr{row_i}"));
                }
            }
            for (i, id) in row_ids.iter().enumerate() {
                term.set_character_visibility(*id, true);
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    ch.animation.activate_scene("print");
                    ch.motion.activate_path("home");
                }
                typed.push(*id);
                if let Some(ch) = term.get_characters().iter().find(|c| c.id == *id) {
                    let target = Coord::new(ch.input_coord.column + 1, ch.input_coord.row);
                    if let Some(head) = term.get_characters_mut().iter_mut().find(|c| c.id == head_id)
                    {
                        head.motion.current_coord = target;
                        head.current_coord = target;
                    }
                }
                for _ in 0..3 {
                    term.step_all();
                    frames.push(render_ansi(&term, head_id));
                }
                if i + 1 == row_ids.len() {
                    for tid in row_ids {
                        if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *tid)
                        {
                            ch.animation.activate_scene("final");
                        }
                    }
                }
            }
        }

        term.set_character_visibility(head_id, false);
        for _ in 0..8 {
            term.step_all();
            frames.push(render_ansi(&term, head_id));
        }
        if frames.is_empty() {
            frames.push(render_ansi(&term, head_id));
        }
        frames
    }
}

fn sgr(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

fn render_ansi(term: &Terminal, head_id: CharacterId) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<(char, Option<Color>)>> = vec![vec![(' ', None); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x < 0 || y < 0 {
            continue;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= w || y >= h {
            continue;
        }
        let sym = ch.animation.current_character_visual.symbol;
        let col = ch
            .animation
            .current_character_visual
            .colors
            .and_then(|p| p.fg)
            .or_else(|| ch.colors.and_then(|p| p.fg));
        if ch.id == head_id {
            grid[y][x] = (sym, Some(Color::rgb(255, 240, 180)));
        } else {
            grid[y][x] = (sym, col);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, col) in row {
            if let Some(c) = col {
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
