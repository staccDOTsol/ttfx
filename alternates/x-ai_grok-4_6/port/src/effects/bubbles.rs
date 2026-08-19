use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Bubbles;

impl Bubbles {
    pub fn new() -> Self {
        Self
    }
}

struct Bubble {
    chars: Vec<CharacterId>,
    anchor: Coord,
    radius: i32,
    color: Color,
}

impl Effect for Bubbles {
    fn name(&self) -> &str {
        "bubbles"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut w = 80usize;
        let mut h = 24usize;
        let lines: Vec<&str> = input.lines().collect();
        if !lines.is_empty() {
            h = lines.len().max(1);
            w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        }
        let mut term = Terminal::from_input(input, w, h);
        if term.get_characters().is_empty() {
            return Vec::new();
        }

        let bubble_stops = vec![
            Color::from_hex("d1f0fa").unwrap_or(Color::rgb(209, 240, 250)),
            Color::from_hex("00d1ff").unwrap_or(Color::rgb(0, 209, 255)),
            Color::from_hex("0078ff").unwrap_or(Color::rgb(0, 120, 255)),
            Color::from_hex("8a2be2").unwrap_or(Color::rgb(138, 43, 226)),
        ];
        let sheen_grad = Gradient::new(bubble_stops, 8);
        let sheen_colors = sheen_grad.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        let n = ids.len();
        let group = ((n as f64).sqrt() as usize).clamp(4, 16);
        let mut bubbles: Vec<Bubble> = Vec::new();
        let mut i = 0;
        let mut color_i = 0;
        while i < n {
            let end = (i + group).min(n);
            let slice = &ids[i..end];
            let mut sx = 0i32;
            let mut sy = 0i32;
            for id in slice {
                if let Some(ch) = term.get_characters().iter().find(|c| c.id == *id) {
                    sx += ch.input_coord.column;
                    sy += ch.input_coord.row;
                }
            }
            let cnt = slice.len() as i32;
            let anchor = Coord::new(sx / cnt.max(1), sy / cnt.max(1));
            let radius = ((slice.len() as f64).sqrt() as i32).max(2);
            let color = sheen_colors[color_i % sheen_colors.len()];
            color_i += 1;
            bubbles.push(Bubble {
                chars: slice.to_vec(),
                anchor,
                radius,
                color,
            });
            i = end;
        }

        for b in &bubbles {
            let ring = find_coords_on_circle(b.anchor, b.radius, b.chars.len());
            for (idx, id) in b.chars.iter().enumerate() {
                let dest = ring.get(idx).copied().unwrap_or(b.anchor);
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    let scn = ch.animation.new_scene("sheen");
                    scn.is_looping = true;
                    for c in &sheen_colors {
                        scn.add_frame(
                            ch.input_symbol,
                            2,
                            Some(ColorPair {
                                fg: Some(*c),
                                bg: None,
                            }),
                        );
                    }
                    scn.add_frame(
                        ch.input_symbol,
                        4,
                        Some(ColorPair {
                            fg: Some(b.color),
                            bg: None,
                        }),
                    );
                    ch.animation.activate_scene("sheen");
                    let p = ch.motion.new_path("form");
                    p.speed = 0.35;
                    p.easing = Easing::OutSine;
                    p.new_waypoint("c", dest);
                }
                term.set_character_visibility(*id, true);
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    ch.motion.activate_path("form");
                }
            }
        }

        let mut frames = Vec::new();
        for _ in 0..36 {
            term.step_all();
            frames.push(colorize_frame(&term, &sheen_colors));
        }

        for b in &bubbles {
            let pop = find_coords_on_circle(b.anchor, b.radius + 3, b.chars.len());
            for (idx, id) in b.chars.iter().enumerate() {
                let dest = pop.get(idx).copied().unwrap_or(b.anchor);
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    let scn = ch.animation.new_scene("pop");
                    scn.add_frame(
                        '*',
                        2,
                        Some(ColorPair {
                            fg: Some(Color::rgb(255, 255, 255)),
                            bg: None,
                        }),
                    );
                    scn.add_frame(
                        ch.input_symbol,
                        3,
                        Some(ColorPair {
                            fg: Some(b.color),
                            bg: None,
                        }),
                    );
                    ch.animation.activate_scene("pop");
                    let p = ch.motion.new_path("pop_out");
                    p.speed = 0.3;
                    p.easing = Easing::OutCubic;
                    p.new_waypoint("p", dest);
                    ch.motion.activate_path("pop_out");
                }
            }
        }
        for _ in 0..18 {
            term.step_all();
            frames.push(colorize_frame(&term, &sheen_colors));
        }

        for id in &ids {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let home = ch.input_coord;
                let scn = ch.animation.new_scene("home");
                let final_c = sheen_colors[id.0 as usize % sheen_colors.len()];
                scn.add_frame(
                    ch.input_symbol,
                    8,
                    Some(ColorPair {
                        fg: Some(final_c),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("home");
                let p = ch.motion.new_path("home");
                p.speed = 0.45;
                p.easing = Easing::InOutSine;
                p.new_waypoint("h", home);
                ch.motion.activate_path("home");
            }
        }
        for _ in 0..40 {
            term.step_all();
            frames.push(colorize_frame(&term, &sheen_colors));
        }

        frames
    }
}

fn colorize_frame(term: &Terminal, palette: &[Color]) -> String {
    let raw = {
        let mut t = Terminal::new(term.canvas.width, term.canvas.height);
        t.canvas = term.canvas.clone();
        // rebuild from current character positions
        let _ = t;
        String::new()
    };
    let _ = raw;
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid = vec![(' ', None::<Color>); w * h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let idx = y as usize * w + x as usize;
            let col = ch
                .animation
                .current_character_visual
                .colors
                .and_then(|p| p.fg)
                .or_else(|| {
                    palette
                        .get(ch.id.0 as usize % palette.len().max(1))
                        .copied()
                });
            grid[idx] = (ch.animation.current_character_visual.symbol, col);
        }
    }
    let mut out = String::with_capacity(w * h * 12);
    for y in 0..h {
        for x in 0..w {
            let (sym, col) = grid[y * w + x];
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
