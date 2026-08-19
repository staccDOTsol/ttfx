use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Swarm;

impl Swarm {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Swarm {
    fn name(&self) -> &str {
        "swarm"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let width = input
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = input.lines().count().max(1);
        let mut term = Terminal::from_input(input, width, height);

        let swarm_stops = vec![
            Color::from_hex("e81416").unwrap_or(Color::rgb(232, 20, 22)),
            Color::from_hex("ffa500").unwrap_or(Color::rgb(255, 165, 0)),
            Color::from_hex("faeb36").unwrap_or(Color::rgb(250, 235, 54)),
            Color::from_hex("79c314").unwrap_or(Color::rgb(121, 195, 20)),
            Color::from_hex("487de7").unwrap_or(Color::rgb(72, 125, 231)),
            Color::from_hex("4b369d").unwrap_or(Color::rgb(75, 54, 157)),
            Color::from_hex("70369d").unwrap_or(Color::rgb(112, 54, 157)),
        ];
        let swarm_grad = Gradient::new(swarm_stops, 24);
        let swarm_colors = swarm_grad.colors();
        let final_grad = Gradient::new(
            vec![
                Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
                Color::from_hex("00D1D1").unwrap_or(Color::rgb(0, 209, 209)),
            ],
            12,
        );
        let final_colors = final_grad.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return vec![term.get_formatted_output_string()];
        }

        let cx = (width as i32) / 2;
        let cy = (height as i32) / 2;
        let n = ids.len();

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let sc = swarm_colors[i % swarm_colors.len()];
                let fc = final_colors[i % final_colors.len()];
                let theta = (i as f64) * std::f64::consts::TAU / (n as f64).max(1.0);
                let r = 2 + ((i as i32) % 5);
                let start = Coord::new(
                    cx + (r as f64 * theta.cos()).round() as i32,
                    cy + (r as f64 * theta.sin()).round() as i32,
                );
                ch.motion.current_coord = start;
                ch.current_coord = start;

                {
                    let scn = ch.animation.new_scene("swarm");
                    scn.is_looping = true;
                    for c in &swarm_colors {
                        scn.add_frame(
                            ch.input_symbol,
                            2,
                            Some(ColorPair {
                                fg: Some(*c),
                                bg: None,
                            }),
                        );
                    }
                }
                {
                    let scn = ch.animation.new_scene("final");
                    scn.add_frame(
                        ch.input_symbol,
                        8,
                        Some(ColorPair {
                            fg: Some(fc),
                            bg: None,
                        }),
                    );
                    scn.add_frame(
                        ch.input_symbol,
                        1,
                        Some(ColorPair {
                            fg: Some(sc),
                            bg: None,
                        }),
                    );
                }
                ch.animation.activate_scene("swarm");
                {
                    let p = ch.motion.new_path("cluster");
                    p.speed = 0.35;
                    p.easing = Easing::InOutSine;
                    p.new_waypoint("c", Coord::new(cx, cy));
                }
                {
                    let p = ch.motion.new_path("home");
                    p.speed = 0.25;
                    p.easing = Easing::OutCubic;
                    p.new_waypoint("h", ch.input_coord);
                }
                ch.motion.activate_path("cluster");
            }
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        let cluster_frames = 40 + n.min(40);
        let home_frames = 50 + n.min(60);

        for t in 0..cluster_frames {
            if t == cluster_frames / 3 {
                for id in &ids {
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                        let jitter = Coord::new(
                            cx + (((id.0 as i32) % 7) - 3),
                            cy + ((((id.0 as i32) / 3) % 5) - 2),
                        );
                        if let Some(p) = ch.motion.new_path("buzz").waypoints.first() {
                            let _ = p;
                        }
                        let p = ch.motion.new_path(format!("buzz{t}"));
                        p.speed = 0.5;
                        p.easing = Easing::InOutQuad;
                        p.new_waypoint("j", jitter);
                        let pid = p.id.clone();
                        ch.motion.activate_path(&pid);
                    }
                }
            }
            term.step_all();
            frames.push(render_ansi(&term));
        }

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                ch.animation.activate_scene("final");
                ch.motion.activate_path("home");
            }
            if i % 3 == 0 {
                term.step_all();
                frames.push(render_ansi(&term));
            }
        }

        for _ in 0..home_frames {
            term.step_all();
            frames.push(render_ansi(&term));
        }

        // settle on input coords with final colors
        for ch in term.get_characters_mut() {
            ch.current_coord = ch.input_coord;
            ch.motion.current_coord = ch.input_coord;
            ch.animation.activate_scene("final");
        }
        for _ in 0..8 {
            term.step_all();
            for ch in term.get_characters_mut() {
                ch.current_coord = ch.input_coord;
            }
            frames.push(render_ansi(&term));
        }
        frames
    }
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<(char, Option<ColorPair>)>> =
        vec![vec![(' ', None); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let vis = &ch.animation.current_character_visual;
            grid[y as usize][x as usize] = (vis.symbol, vis.colors);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, colors) in row {
            if let Some(cp) = colors {
                if let Some(fg) = cp.fg {
                    out.push_str(&format!("\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b));
                }
                if let Some(bg) = cp.bg {
                    out.push_str(&format!("\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b));
                }
                out.push(sym);
                out.push_str("\x1b[0m");
            } else if sym != ' ' {
                out.push_str("\x1b[38;2;138;43;226m");
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
