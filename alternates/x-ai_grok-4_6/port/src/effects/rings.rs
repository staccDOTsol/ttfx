use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Rings;

impl Rings {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Rings {
    fn name(&self) -> &str {
        "rings"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut width = 40usize;
        let mut height = 12usize;
        for (i, line) in input.lines().enumerate() {
            width = width.max(line.chars().count().max(1));
            height = height.max(i + 1);
        }
        height = height.max(1);
        let mut term = Terminal::from_input(input, width, height);
        if term.get_characters().is_empty() {
            return vec!["\n".to_string()];
        }

        let palette = Gradient::new(
            vec![
                Color::from_hex("ff0000").unwrap_or(Color::rgb(255, 0, 0)),
                Color::from_hex("ffaa00").unwrap_or(Color::rgb(255, 170, 0)),
                Color::from_hex("00ff88").unwrap_or(Color::rgb(0, 255, 136)),
                Color::from_hex("4488ff").unwrap_or(Color::rgb(68, 136, 255)),
            ],
            24,
        )
        .colors();

        let cx = (width as i32) / 2;
        let cy = (height as i32) / 2;
        let center = Coord::new(cx, cy);
        let n = term.get_characters().len();
        let ring_gap = 3i32;
        let max_r = cx.max(cy).max(2);

        let mut ring_slots: Vec<Vec<Coord>> = Vec::new();
        for r in (1..=max_r).step_by(ring_gap as usize) {
            let count = (7 * r as usize).max(8);
            ring_slots.push(find_coords_on_circle(center, r, count));
        }
        if ring_slots.is_empty() {
            ring_slots.push(vec![center]);
        }

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for (i, id) in ids.iter().enumerate() {
            let ring = &ring_slots[i % ring_slots.len()];
            let start = ring[i % ring.len()];
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                ch.motion.current_coord = start;
                ch.current_coord = start;
                let color = palette[i % palette.len()];
                let scn = ch.animation.new_scene("spin");
                scn.is_looping = true;
                scn.add_frame(
                    ch.input_symbol,
                    2,
                    Some(ColorPair {
                        fg: Some(color),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("spin");
                let home = ch.animation.new_scene("home");
                home.add_frame(
                    ch.input_symbol,
                    8,
                    Some(ColorPair {
                        fg: Some(palette[(i + 8) % palette.len()]),
                        bg: None,
                    }),
                );
                let p = ch.motion.new_path("orbit");
                p.speed = 0.35;
                p.new_waypoint("o", start);
                let hp = ch.motion.new_path("home");
                hp.speed = 0.55;
                hp.easing = crate::utils::easing::Easing::OutQuad;
                let dest = ch.input_coord;
                hp.new_waypoint("h", dest);
            }
            term.set_character_visibility(*id, true);
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                ch.motion.activate_path("orbit");
            }
        }

        let mut out = Vec::new();
        let spin_frames = 36usize;
        for tick in 0..spin_frames {
            for (i, id) in ids.iter().enumerate() {
                let ring = &ring_slots[i % ring_slots.len()];
                let idx = (i + tick) % ring.len();
                let pos = ring[idx];
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    ch.motion.current_coord = pos;
                    ch.current_coord = pos;
                }
            }
            term.step_all();
            out.push(paint(&term, &palette));
        }

        for id in &ids {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                ch.animation.activate_scene("home");
                ch.motion.activate_path("home");
            }
        }
        for _ in 0..28 {
            term.step_all();
            out.push(paint(&term, &palette));
        }
        out
    }
}

fn paint(term: &Terminal, palette: &[Color]) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid = vec![vec![' '; w]; h];
    let mut cols = vec![vec![None; w]; h];
    for (i, ch) in term.get_characters().iter().enumerate() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let c = ch
                .animation
                .current_character_visual
                .colors
                .and_then(|p| p.fg)
                .unwrap_or(palette[i % palette.len()]);
            grid[y as usize][x as usize] = ch.animation.current_character_visual.symbol;
            cols[y as usize][x as usize] = Some(c);
        }
    }
    let mut s = String::new();
    for y in 0..h {
        for x in 0..w {
            if let Some(c) = cols[y][x] {
                s.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, grid[y][x]));
            } else {
                s.push(grid[y][x]);
            }
        }
        s.push('\n');
    }
    s
}
