use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Binarypath;

impl Binarypath {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Binarypath {
    fn name(&self) -> &str {
        "binarypath"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let final_gradient = Gradient::new(
            vec![
                Color::from_hex("00ffaa").unwrap_or(Color::rgb(0, 255, 170)),
                Color::from_hex("00ddff").unwrap_or(Color::rgb(0, 221, 255)),
                Color::from_hex("0088ff").unwrap_or(Color::rgb(0, 136, 255)),
            ],
            8,
        );
        let binary_gradient = Gradient::new(
            vec![
                Color::from_hex("003300").unwrap_or(Color::rgb(0, 51, 0)),
                Color::from_hex("00ff00").unwrap_or(Color::rgb(0, 255, 0)),
            ],
            6,
        );
        let final_colors = final_gradient.colors();
        let bin_colors = binary_gradient.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return vec![term.get_formatted_output_string()];
        }

        for ch in term.get_characters_mut() {
            let bits: Vec<char> = format!("{:08b}", ch.input_symbol as u32)
                .chars()
                .collect();
            {
                let scn = ch.animation.new_scene("binary");
                scn.is_looping = true;
                for (i, &bit) in bits.iter().enumerate() {
                    let col = bin_colors[i % bin_colors.len()];
                    scn.add_frame(
                        bit,
                        2,
                        Some(ColorPair {
                            fg: Some(col),
                            bg: None,
                        }),
                    );
                }
            }
            {
                let scn = ch.animation.new_scene("final");
                let idx = ((ch.input_coord.column + ch.input_coord.row).unsigned_abs() as usize)
                    % final_colors.len();
                scn.add_frame(
                    ch.input_symbol,
                    1,
                    Some(ColorPair {
                        fg: Some(final_colors[idx]),
                        bg: None,
                    }),
                );
            }
            ch.animation.activate_scene("binary");
            let dest = ch.input_coord;
            let start = Coord::new(0, dest.row);
            ch.motion.current_coord = start;
            ch.current_coord = start;
            {
                let path = ch.motion.new_path("home");
                path.speed = 0.35;
                path.easing = Easing::OutQuad;
                path.new_waypoint("end", dest);
            }
            ch.motion.activate_path("home");
        }

        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        let max_frames = 180usize;
        for tick in 0..max_frames {
            if tick == 90 {
                for ch in term.get_characters_mut() {
                    ch.animation.activate_scene("final");
                }
            }
            term.step_all();
            frames.push(render_ansi(&term));
            let all_home = term.get_characters().iter().all(|c| {
                c.current_coord == c.input_coord && tick > 90
            });
            if all_home && tick > 100 {
                break;
            }
        }
        // hold final styled frame
        for _ in 0..8 {
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
            let vis = &ch.animation.current_character_visual;
            let fg = vis.colors.and_then(|p| p.fg).or_else(|| {
                ch.colors.and_then(|p| p.fg)
            });
            grid[uy][ux] = (vis.symbol, fg);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, fg) in row {
            if let Some(c) = fg {
                out.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, sym));
            } else if sym != ' ' {
                out.push_str("\x1b[38;2;0;255;128m");
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
