use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Fireworks;

impl Fireworks {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Fireworks {
    fn name(&self) -> &str {
        "fireworks"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height.max(8));
        let w = term.canvas.width as i32;
        let h = term.canvas.height as i32;
        if term.get_characters().is_empty() {
            return vec![String::new()];
        }

        let explode_stops = vec![
            Color::rgb(255, 0, 0),
            Color::rgb(255, 180, 0),
            Color::rgb(255, 255, 80),
            Color::rgb(255, 80, 160),
            Color::rgb(80, 160, 255),
        ];
        let explode_grad = Gradient::new(explode_stops, 12);
        let explode_colors = explode_grad.colors();
        let fade_grad = Gradient::new(
            vec![Color::rgb(255, 220, 80), Color::rgb(80, 40, 20), Color::rgb(20, 20, 20)],
            8,
        );
        let fade_colors = fade_grad.colors();

        let launch = Coord::new(w / 2, h.saturating_sub(1).max(0));
        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        let n = ids.len().max(1);

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let home = ch.input_coord;
                ch.motion.current_coord = launch;
                ch.current_coord = launch;
                let burst_r = 2 + ((i as i32) % 5);
                let ring = find_coords_on_circle(home, burst_r, 8.max(n.min(24)));
                let burst = ring[i % ring.len()];
                {
                    let p = ch.motion.new_path("launch");
                    p.speed = 0.35 + (i % 5) as f64 * 0.05;
                    p.easing = Easing::OutCubic;
                    p.new_waypoint("apex", Coord::new(home.column, (home.row / 2).max(0)));
                }
                {
                    let p = ch.motion.new_path("burst");
                    p.speed = 0.25 + (i % 4) as f64 * 0.04;
                    p.easing = Easing::OutQuad;
                    p.new_waypoint("burst", burst);
                }
                {
                    let p = ch.motion.new_path("home");
                    p.speed = 0.18;
                    p.easing = Easing::InQuad;
                    p.new_waypoint("home", home);
                }
                let col = explode_colors[i % explode_colors.len()];
                {
                    let scn = ch.animation.new_scene("spark");
                    for c in explode_colors.iter().cycle().take(10) {
                        scn.add_frame(
                            '*',
                            2,
                            Some(ColorPair {
                                fg: Some(*c),
                                bg: None,
                            }),
                        );
                    }
                    scn.is_looping = true;
                }
                {
                    let scn = ch.animation.new_scene("settle");
                    for c in &fade_colors {
                        scn.add_frame(
                            ch.input_symbol,
                            3,
                            Some(ColorPair {
                                fg: Some(*c),
                                bg: None,
                            }),
                        );
                    }
                    scn.add_frame(
                        ch.input_symbol,
                        8,
                        Some(ColorPair {
                            fg: Some(col),
                            bg: None,
                        }),
                    );
                }
                ch.animation.set_appearance(
                    '*',
                    Some(explode_colors[i % explode_colors.len()]),
                );
                ch.animation.activate_scene("spark");
                ch.motion.activate_path("launch");
            }
            term.set_character_visibility(*id, true);
        }

        let mut out = Vec::new();
        let total = 90 + n.min(40);
        for tick in 0..total {
            if tick == 18 {
                for id in &ids {
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                        ch.motion.activate_path("burst");
                    }
                }
            }
            if tick == 48 {
                for id in &ids {
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                        ch.motion.activate_path("home");
                        ch.animation.activate_scene("settle");
                    }
                }
            }
            term.step_all();
            out.push(render_ansi(&term));
        }
        out
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
        if x < 0 || y < 0 {
            continue;
        }
        let (x, y) = (x as usize, y as usize);
        if x < w && y < h {
            let sym = ch.animation.current_character_visual.symbol;
            let col = ch
                .animation
                .current_character_visual
                .colors
                .and_then(|p| p.fg)
                .or_else(|| ch.colors.and_then(|p| p.fg));
            grid[y][x] = (sym, col);
        }
    }
    let mut s = String::new();
    for row in grid {
        for (sym, col) in row {
            if let Some(c) = col {
                s.push_str(&sgr(c));
                s.push(sym);
                s.push_str("\x1b[0m");
            } else {
                s.push(sym);
            }
        }
        s.push('\n');
    }
    s
}
