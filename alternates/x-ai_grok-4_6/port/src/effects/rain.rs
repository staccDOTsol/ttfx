use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Rain;

impl Rain {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Rain {
    fn name(&self) -> &str {
        "rain"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let rain_stops = vec![
            Color::from_hex("00315C").unwrap_or(Color::rgb(0, 49, 92)),
            Color::from_hex("03ABFC").unwrap_or(Color::rgb(3, 171, 252)),
        ];
        let rain_grad = Gradient::new(rain_stops, 8);
        let rain_colors = rain_grad.colors();

        let final_stops = vec![
            Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
            Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
            Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let final_grad = Gradient::new(final_stops, 12);
        let final_colors = final_grad.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        let n = ids.len().max(1);

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let rain_c = rain_colors[i % rain_colors.len()];
                let fin_c = {
                    let span = (width.max(1) + height.max(1)) as i32;
                    let t = ((ch.input_coord.column + ch.input_coord.row).max(0) as usize)
                        % final_colors.len().max(1);
                    let _ = span;
                    final_colors[t]
                };
                let start = Coord::new(ch.input_coord.column, -(i as i32 % 8) - 1);
                ch.motion.current_coord = start;
                ch.current_coord = start;
                {
                    let scn = ch.animation.new_scene("raindrop");
                    scn.add_frame(ch.input_symbol, 1, Some(ColorPair { fg: Some(rain_c), bg: None }));
                }
                {
                    let scn = ch.animation.new_scene("final");
                    scn.add_frame(ch.input_symbol, 1, Some(ColorPair { fg: Some(fin_c), bg: None }));
                }
                ch.animation.activate_scene("raindrop");
                let path = ch.motion.new_path("fall");
                path.speed = 0.35 + ((i as f64 * 0.17) % 0.45);
                path.easing = Easing::InQuad;
                path.new_waypoint("home", ch.input_coord);
                ch.motion.activate_path("fall");
            }
        }

        let mut pending: Vec<CharacterId> = ids.clone();
        pending.sort_by_key(|id| {
            term.get_characters()
                .iter()
                .find(|c| c.id == *id)
                .map(|c| (c.input_coord.column, c.input_coord.row))
                .unwrap_or((0, 0))
        });

        let mut active: Vec<CharacterId> = Vec::new();
        let mut frames = Vec::new();
        let mut tick = 0usize;
        let max_frames = 400 + n * 2;

        while tick < max_frames {
            if !pending.is_empty() && tick % 2 == 0 {
                let batch = ((n / 12).max(1)).min(pending.len());
                for _ in 0..batch {
                    let id = pending.remove(0);
                    term.set_character_visibility(id, true);
                    active.push(id);
                }
            }

            term.step_all();

            let mut still = Vec::new();
            for id in active.drain(..) {
                let done = term
                    .get_characters()
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.current_coord == c.input_coord)
                    .unwrap_or(true);
                if done {
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == id) {
                        ch.animation.activate_scene("final");
                        ch.animation.step_animation();
                    }
                } else {
                    still.push(id);
                }
            }
            active = still;

            frames.push(render_ansi(&term));
            tick += 1;
            if pending.is_empty() && active.is_empty() {
                frames.push(render_ansi(&term));
                break;
            }
        }
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
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let fg = ch
                .animation
                .current_character_visual
                .colors
                .and_then(|p| p.fg)
                .or_else(|| ch.colors.and_then(|p| p.fg));
            grid[y as usize][x as usize] = (ch.animation.current_character_visual.symbol, fg);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, fg) in row {
            if let Some(c) = fg {
                out.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, sym));
            } else if sym != ' ' {
                out.push_str("\x1b[38;2;3;171;252m");
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
