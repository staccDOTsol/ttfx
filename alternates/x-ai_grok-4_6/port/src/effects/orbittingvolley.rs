use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Orbittingvolley;

impl Orbittingvolley {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Orbittingvolley {
    fn name(&self) -> &str {
        "orbittingvolley"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height.max(1));

        let top = 0i32;
        let bottom = (height.saturating_sub(1)) as i32;
        let left = 0i32;
        let right = (width.saturating_sub(1)) as i32;

        let stops = vec![
            Color::from_hex("ff6600").unwrap_or(Color::rgb(255, 102, 0)),
            Color::from_hex("ffff00").unwrap_or(Color::rgb(255, 255, 0)),
            Color::from_hex("00ff88").unwrap_or(Color::rgb(0, 255, 136)),
            Color::from_hex("0088ff").unwrap_or(Color::rgb(0, 136, 255)),
            Color::from_hex("ff00aa").unwrap_or(Color::rgb(255, 0, 170)),
        ];
        let gradient = Gradient::new(stops, 24.max(term.get_characters().len().max(1)));
        let palette = gradient.colors();

        let perimeter = [
            Coord::new(left, top),
            Coord::new(right, top),
            Coord::new(right, bottom),
            Coord::new(left, bottom),
        ];

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for (i, id) in ids.iter().enumerate() {
            let color = palette[i % palette.len()];
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let pair = ColorPair {
                    fg: Some(color),
                    bg: None,
                };
                ch.colors = Some(pair);
                let scn = ch.animation.new_scene("volley");
                scn.add_frame(ch.input_symbol, 1, Some(pair));
                ch.animation.activate_scene("volley");

                let launch = perimeter[i % perimeter.len()];
                ch.motion.current_coord = launch;
                ch.current_coord = launch;

                let path = ch.motion.new_path("input_path");
                path.speed = 0.35 + ((i % 5) as f64) * 0.08;
                path.easing = Easing::OutQuad;
                path.new_waypoint("home", ch.input_coord);
            }
            term.set_character_visibility(*id, false);
        }

        // orbiting launchers (visual markers)
        let launcher_ids: Vec<CharacterId> = perimeter
            .iter()
            .map(|&p| term.add_character('*', p))
            .collect();
        for (li, lid) in launcher_ids.iter().enumerate() {
            let lc = Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255));
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *lid) {
                let pair = ColorPair {
                    fg: Some(lc),
                    bg: None,
                };
                let scn = ch.animation.new_scene("orbit");
                scn.is_looping = true;
                scn.add_frame('*', 2, Some(pair));
                scn.add_frame('+', 2, Some(pair));
                ch.animation.activate_scene("orbit");
                let path = ch.motion.new_path("perimeter");
                path.speed = 0.2;
                path.easing = Easing::Linear;
                for (wi, wp) in perimeter.iter().cycle().skip(li).take(5).enumerate() {
                    path.new_waypoint(format!("w{wi}"), *wp);
                }
                ch.motion.activate_path("perimeter");
            }
            term.set_character_visibility(*lid, true);
        }

        let mut frames = Vec::new();
        let mut launched = 0usize;
        let mut tick = 0usize;
        let total = ids.len();
        let max_frames = 240 + total * 4;

        while tick < max_frames {
            if launched < total && tick % 3 == 0 {
                let id = ids[launched];
                term.set_character_visibility(id, true);
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == id) {
                    ch.motion.activate_path("input_path");
                }
                launched += 1;
            }

            term.step_all();
            frames.push(render_ansi(&term));

            if launched >= total {
                let all_home = ids.iter().all(|id| {
                    term.get_characters()
                        .iter()
                        .find(|c| c.id == *id)
                        .map(|c| c.current_coord == c.input_coord)
                        .unwrap_or(false)
                });
                if all_home && tick > total * 3 + 8 {
                    break;
                }
            }
            tick += 1;
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
    let mut grid: Vec<Vec<(char, Option<ColorPair>)>> = vec![vec![(' ', None); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let sym = ch.animation.current_character_visual.symbol;
            let colors = ch
                .animation
                .current_character_visual
                .colors
                .or(ch.colors);
            grid[y as usize][x as usize] = (sym, colors);
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
                out.push_str("\x1b[38;2;200;200;200m");
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
