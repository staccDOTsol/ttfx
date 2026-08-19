use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Middleout;

impl Middleout {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Middleout {
    fn name(&self) -> &str {
        "middleout"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            lines.push("");
        }
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);

        let mut term = Terminal::from_input(input, width, height);
        let center_col = (width as i32) / 2;
        let center_row = (height as i32) / 2;

        let gradient = Gradient::new(
            vec![
                Color::from_hex("88c0d0").unwrap_or(Color::rgb(0x88, 0xc0, 0xd0)),
                Color::from_hex("5e81ac").unwrap_or(Color::rgb(0x5e, 0x81, 0xac)),
                Color::from_hex("b48ead").unwrap_or(Color::rgb(0xb4, 0x8e, 0xad)),
                Color::from_hex("ebcb8b").unwrap_or(Color::rgb(0xeb, 0xcb, 0x8b)),
            ],
            12,
        );
        let palette = gradient.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let idx = ((ch.input_coord.column.abs() + ch.input_coord.row.abs()) as usize)
                    % palette.len().max(1);
                let color = palette.get(idx).copied().unwrap_or(Color::rgb(200, 200, 220));
                let pair = ColorPair {
                    fg: Some(color),
                    bg: None,
                };
                ch.colors = Some(pair);
                let scn = ch.animation.new_scene("mid");
                scn.add_frame(ch.input_symbol, 2, Some(pair));
                ch.animation.activate_scene("mid");

                let dest_center = Coord::new(ch.input_coord.column, center_row);
                // start at the middle of the same column (vertical expand)
                ch.motion.current_coord = dest_center;
                ch.current_coord = dest_center;

                let path = ch.motion.new_path("full");
                path.speed = 0.35;
                path.easing = Easing::OutCubic;
                path.new_waypoint("home", ch.input_coord);
                ch.motion.activate_path("full");
            }
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        for _ in 0..48 {
            term.step_all();
            frames.push(render_ansi(&term, &palette, center_col, center_row));
        }
        if frames.is_empty() {
            frames.push(render_ansi(&term, &palette, center_col, center_row));
        }
        frames
    }
}

fn render_ansi(term: &Terminal, palette: &[Color], _cx: i32, _cy: i32) -> String {
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
        let ux = x as usize;
        let uy = y as usize;
        if ux < w && uy < h {
            let color = ch
                .animation
                .current_character_visual
                .colors
                .and_then(|p| p.fg)
                .or_else(|| ch.colors.and_then(|p| p.fg))
                .or_else(|| palette.first().copied());
            grid[uy][ux] = (ch.animation.current_character_visual.symbol, color);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, col) in row {
            if let Some(c) = col {
                out.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, sym));
            } else {
                out.push(sym);
            }
        }
        out.push('\n');
    }
    out
}
