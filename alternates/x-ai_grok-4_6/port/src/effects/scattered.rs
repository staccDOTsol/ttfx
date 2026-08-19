use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Scattered;

impl Scattered {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Scattered {
    fn name(&self) -> &str {
        "scattered"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let gradient = Gradient::new(
            vec![
                Color::from_hex("88c0d0").unwrap_or(Color::rgb(0x88, 0xc0, 0xd0)),
                Color::from_hex("81a1c1").unwrap_or(Color::rgb(0x81, 0xa1, 0xc1)),
                Color::from_hex("5e81ac").unwrap_or(Color::rgb(0x5e, 0x81, 0xac)),
            ],
            8,
        );
        let palette = gradient.colors();
        let n_colors = palette.len().max(1);

        let mut rng = 0xC0FFEE_u64;
        let mut next_rand = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            rng
        };

        let w = width as i32;
        let h = height as i32;
        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let rx = (next_rand() as i32).rem_euclid(w.max(1));
                let ry = (next_rand() as i32).rem_euclid(h.max(1));
                let start = Coord::new(rx, ry);
                ch.motion.current_coord = start;
                ch.current_coord = start;
                let color = palette[i % n_colors];
                let scn = ch.animation.new_scene("scatter");
                scn.add_frame(
                    ch.input_symbol,
                    1,
                    Some(ColorPair {
                        fg: Some(color),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("scatter");
                ch.animation.set_appearance(ch.input_symbol, Some(color));
                let home = ch.input_coord;
                let path = ch.motion.new_path("home");
                path.speed = 0.35;
                path.easing = Easing::OutQuad;
                path.new_waypoint("dest", home);
                ch.motion.activate_path("home");
            }
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        let max_frames = 240usize;
        for _ in 0..max_frames {
            frames.push(render_ansi(&term));
            let settled = term.get_characters().iter().all(|c| {
                c.current_coord.column == c.input_coord.column
                    && c.current_coord.row == c.input_coord.row
            });
            term.step_all();
            if settled {
                break;
            }
        }
        frames.push(render_ansi(&term));
        if frames.is_empty() {
            frames.push(render_ansi(&term));
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
                .or_else(|| ch.colors.and_then(|p| p.fg));
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
