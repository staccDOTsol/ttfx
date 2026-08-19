use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Unstable;

impl Unstable {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Unstable {
    fn name(&self) -> &str {
        "unstable"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height.max(1) + 2);

        let rumble_stops = vec![
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
            Color::from_hex("ff0000").unwrap_or(Color::rgb(255, 0, 0)),
            Color::from_hex("ffff00").unwrap_or(Color::rgb(255, 255, 0)),
        ];
        let rumble_grad = Gradient::new(rumble_stops, 8).colors();
        let final_stops = vec![
            Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
            Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
            Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let final_grad = Gradient::new(final_stops, 12).colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return vec![sgr_frame(" ", Color::rgb(255, 0, 0))];
        }

        let n = ids.len();
        let mut origins: Vec<Coord> = Vec::new();
        let mut symbols: Vec<char> = Vec::new();
        for ch in term.get_characters() {
            origins.push(ch.input_coord);
            symbols.push(ch.input_symbol);
        }

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let rumble = ch.animation.new_scene("rumble");
                rumble.is_looping = true;
                for c in &rumble_grad {
                    rumble.add_frame(
                        ch.input_symbol,
                        2,
                        Some(ColorPair {
                            fg: Some(*c),
                            bg: None,
                        }),
                    );
                }
                let home = ch.animation.new_scene("home");
                let fc = final_grad[i % final_grad.len()];
                home.add_frame(
                    ch.input_symbol,
                    1,
                    Some(ColorPair {
                        fg: Some(fc),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("rumble");
            }
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        // rumble in place
        for _ in 0..18 {
            term.step_all();
            frames.push(render_ansi(&term));
        }

        // explode toward canvas edges
        for (i, id) in ids.iter().enumerate() {
            let dest = match i % 4 {
                0 => Coord::new(0, origins[i].row),
                1 => Coord::new((width as i32).saturating_sub(1), origins[i].row),
                2 => Coord::new(origins[i].column, 0),
                _ => Coord::new(origins[i].column, (height as i32).saturating_sub(1)),
            };
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let p = ch.motion.new_path("explosion");
                p.speed = 0.35;
                p.easing = Easing::OutCubic;
                p.new_waypoint("edge", dest);
                ch.motion.activate_path("explosion");
            }
        }
        for _ in 0..28 {
            term.step_all();
            frames.push(render_ansi(&term));
        }

        // reassemble
        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let p = ch.motion.new_path("reassembly");
                p.speed = 0.25;
                p.easing = Easing::InOutQuad;
                p.new_waypoint("home", origins[i]);
                ch.motion.activate_path("reassembly");
                ch.animation.activate_scene("home");
            }
        }
        for _ in 0..36 {
            term.step_all();
            frames.push(render_ansi(&term));
        }

        // hold final
        for _ in 0..8 {
            frames.push(render_ansi(&term));
        }

        if frames.is_empty() {
            frames.push(sgr_frame(&symbols.iter().collect::<String>(), Color::rgb(0, 209, 255)));
        }
        frames
    }
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width.max(1);
    let h = term.canvas.height.max(1);
    let mut grid = vec![vec![' '; w]; h];
    let mut cols = vec![vec![None; w]; h];
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
                grid[uy][ux] = ch.animation.current_character_visual.symbol;
                cols[uy][ux] = ch
                    .animation
                    .current_character_visual
                    .colors
                    .and_then(|p| p.fg)
                    .or(ch.colors.and_then(|p| p.fg));
            }
        }
    }
    let mut out = String::new();
    for y in 0..h {
        for x in 0..w {
            let c = grid[y][x];
            if let Some(col) = cols[y][x] {
                out.push_str(&format!(
                    "\x1b[38;2;{};{};{}m{}\x1b[0m",
                    col.r, col.g, col.b, c
                ));
            } else if c != ' ' {
                out.push_str(&format!("\x1b[38;2;255;80;80m{c}\x1b[0m"));
            } else {
                out.push(' ');
            }
        }
        out.push('\n');
    }
    out
}

fn sgr_frame(s: &str, c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m{s}\x1b[0m\n", c.r, c.g, c.b)
}
