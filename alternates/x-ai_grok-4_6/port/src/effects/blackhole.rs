use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Blackhole;

impl Blackhole {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Blackhole {
    fn name(&self) -> &str {
        "blackhole"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut w = 80usize;
        let mut h = 24usize;
        let lines: Vec<&str> = input.lines().collect();
        if !lines.is_empty() {
            h = lines.len().max(8);
            w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(40).max(20);
        }
        let mut term = Terminal::from_input(input, w, h);
        if term.get_characters().is_empty() {
            return Vec::new();
        }

        let blackhole_color = Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255));
        let star_stops = vec![
            Color::from_hex("4b0082").unwrap_or(Color::rgb(75, 0, 130)),
            Color::from_hex("8a2be2").unwrap_or(Color::rgb(138, 43, 226)),
            Color::from_hex("00d4ff").unwrap_or(Color::rgb(0, 212, 255)),
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let star_grad = Gradient::new(star_stops, 12);
        let star_colors = star_grad.colors();
        let star_symbols = ['*', '.', '+', 'o', '·'];

        let n = term.get_characters().len();
        let radius = ((w.min(h) as i32) / 6).max(2);
        let center = Coord::new((w as i32) / 2, (h as i32) / 2);
        let ring_n = n.min((radius as usize * 6).max(8));
        let ring_pos = find_coords_on_circle(center, radius, ring_n);

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        let blackhole_ids: Vec<CharacterId> = ids.iter().take(ring_n.min(ids.len())).copied().collect();

        for (i, ch) in term.get_characters_mut().iter_mut().enumerate() {
            let is_bh = i < blackhole_ids.len();
            if is_bh {
                let pos = ring_pos[i % ring_pos.len()];
                let p = ch.motion.new_path("blackhole");
                p.speed = 0.7;
                p.easing = Easing::InOutSine;
                p.new_waypoint("start", pos);
                let scn = ch.animation.new_scene("blackhole");
                scn.add_frame(
                    '*',
                    2,
                    Some(ColorPair {
                        fg: Some(blackhole_color),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("blackhole");
                let rot = ch.motion.new_path("blackhole_rotation");
                rot.speed = 0.45;
                rot.easing = Easing::Linear;
                for (wi, coord) in ring_pos[i % ring_pos.len()..]
                    .iter()
                    .chain(ring_pos[..i % ring_pos.len()].iter())
                    .enumerate()
                {
                    rot.new_waypoint(format!("w{wi}"), *coord);
                }
            } else {
                let scn = ch.animation.new_scene("star");
                let sym = star_symbols[i % star_symbols.len()];
                let col = star_colors[i % star_colors.len()];
                scn.add_frame(
                    sym,
                    3,
                    Some(ColorPair {
                        fg: Some(col),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("star");
                let sx = ((i * 17) % w) as i32;
                let sy = ((i * 13) % h) as i32;
                ch.motion.current_coord = Coord::new(sx, sy);
                ch.current_coord = Coord::new(sx, sy);
                let p = ch.motion.new_path("singularity");
                p.speed = 0.2 + ((i % 13) as f64) * 0.01;
                p.easing = Easing::InCubic;
                p.new_waypoint("c", center);
            }
        }

        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        // collapse toward center / form ring
        for tick in 0..90 {
            for ch in term.get_characters_mut() {
                if blackhole_ids.iter().any(|id| *id == ch.id) {
                    if tick == 0 {
                        ch.motion.activate_path("blackhole");
                    }
                    if tick == 25 {
                        ch.motion.activate_path("blackhole_rotation");
                    }
                } else if tick == 8 {
                    ch.motion.activate_path("singularity");
                }
            }
            term.step_all();
            frames.push(paint(&term, &star_colors, blackhole_color, &blackhole_ids));
        }

        // hold rotation + ingest remaining
        for _ in 0..40 {
            term.step_all();
            frames.push(paint(&term, &star_colors, blackhole_color, &blackhole_ids));
        }

        // expand / restore input glyphs with gradient
        let restore_stops = vec![
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
            Color::from_hex("8a2be2").unwrap_or(Color::rgb(138, 43, 226)),
            Color::from_hex("00d4ff").unwrap_or(Color::rgb(0, 212, 255)),
        ];
        let restore = Gradient::new(restore_stops, 8).colors();
        for ch in term.get_characters_mut() {
            let home = ch.motion.new_path("home");
            home.speed = 0.35;
            home.easing = Easing::OutCubic;
            home.new_waypoint("in", ch.input_coord);
            ch.motion.activate_path("home");
            let scn = ch.animation.new_scene("final");
            for (fi, c) in restore.iter().enumerate() {
                scn.add_frame(
                    ch.input_symbol,
                    2 + fi as u32,
                    Some(ColorPair {
                        fg: Some(*c),
                        bg: None,
                    }),
                );
            }
            ch.animation.activate_scene("final");
        }
        for _ in 0..50 {
            term.step_all();
            frames.push(paint(&term, &restore, blackhole_color, &[]));
        }

        frames
    }
}

fn paint(
    term: &Terminal,
    palette: &[Color],
    hole: Color,
    hole_ids: &[CharacterId],
) -> String {
    let mut out = String::new();
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<Option<(char, Color)>>> = vec![vec![None; w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x < 0 || y < 0 {
            continue;
        }
        let (ux, uy) = (x as usize, y as usize);
        if ux >= w || uy >= h {
            continue;
        }
        let col = if hole_ids.iter().any(|id| *id == ch.id) {
            hole
        } else if let Some(cp) = ch.animation.current_character_visual.colors {
            cp.fg.unwrap_or(palette[0])
        } else {
            palette[(ch.id.0 as usize) % palette.len().max(1)]
        };
        let sym = ch.animation.current_character_visual.symbol;
        grid[uy][ux] = Some((sym, col));
    }
    for row in grid {
        for cell in row {
            match cell {
                Some((sym, c)) => {
                    out.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, sym));
                }
                None => out.push(' '),
            }
        }
        out.push('\n');
    }
    out
}
