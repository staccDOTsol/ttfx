use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Crumble;

impl Crumble {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Crumble {
    fn name(&self) -> &str {
        "crumble"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height.max(4));

        let weaken_stops = vec![
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
            Color::from_hex("444444").unwrap_or(Color::rgb(68, 68, 68)),
            Color::from_hex("888888").unwrap_or(Color::rgb(136, 136, 136)),
        ];
        let weaken_grad = Gradient::new(weaken_stops, 8);
        let weaken_colors = weaken_grad.colors();

        let restore_stops = vec![
            Color::from_hex("888888").unwrap_or(Color::rgb(136, 136, 136)),
            Color::from_hex("00d7ff").unwrap_or(Color::rgb(0, 215, 255)),
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let restore_grad = Gradient::new(restore_stops, 6);
        let restore_colors = restore_grad.colors();

        let bottom = term.canvas.height.saturating_sub(1) as i32;
        let top = 0i32;
        let center_col = (term.canvas.width as i32) / 2;
        let center_row = (term.canvas.height as i32) / 2;

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        for ch in term.get_characters_mut() {
            let weaken = ch.animation.new_scene("weaken");
            for (i, col) in weaken_colors.iter().enumerate() {
                weaken.add_frame(
                    ch.input_symbol,
                    if i == 0 { 3 } else { 2 },
                    Some(ColorPair {
                        fg: Some(*col),
                        bg: None,
                    }),
                );
            }
            let bounce = ch.animation.new_scene("bounce");
            for col in &restore_colors {
                bounce.add_frame(
                    ch.input_symbol,
                    2,
                    Some(ColorPair {
                        fg: Some(*col),
                        bg: None,
                    }),
                );
            }
            bounce.add_frame(
                ch.input_symbol,
                4,
                Some(ColorPair {
                    fg: Some(Color::rgb(255, 255, 255)),
                    bg: None,
                }),
            );

            let fall = ch.motion.new_path("fall");
            fall.speed = 0.35 + ((ch.id.0 as f64) * 0.017) % 0.45;
            fall.easing = Easing::OutCubic;
            let _ = fall.new_waypoint("bot", Coord::new(ch.input_coord.column, bottom));

            let lift = ch.motion.new_path("top");
            lift.speed = 0.55;
            lift.easing = Easing::OutSine;
            let _ = lift.new_waypoint(
                "mid",
                Coord::new(center_col, center_row),
            );
            let _ = lift.new_waypoint("apex", Coord::new(ch.input_coord.column, top));

            let home = ch.motion.new_path("home");
            home.speed = 0.4;
            home.easing = Easing::InOutQuad;
            let _ = home.new_waypoint("in", ch.input_coord);

            ch.animation.activate_scene("weaken");
        }

        let n = ids.len();
        let mut frames = Vec::new();
        let total = (n as i32 * 3 + 80).max(90) as usize;

        for tick in 0..total {
            if tick % 3 == 0 {
                let start = (tick / 3).min(n.saturating_sub(1));
                if start < n {
                    let id = ids[start];
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == id) {
                        if ch.motion.active_path.is_none() {
                            ch.motion.activate_path("fall");
                            ch.animation.activate_scene("weaken");
                        }
                    }
                }
            }

            for ch in term.get_characters_mut() {
                if ch.motion.active_path.as_deref() == Some("fall") {
                    if let Some(p) = ch.motion.query_path("fall") {
                        if (ch.motion.current_coord.row - bottom).abs() <= 0 || p.speed < 0.0 {
                            // path progress is internal; bounce when near bottom
                        }
                    }
                    if ch.motion.current_coord.row >= bottom.saturating_sub(0)
                        && tick > 8
                    {
                        ch.animation.activate_scene("bounce");
                        ch.motion.activate_path("top");
                    }
                } else if ch.motion.active_path.as_deref() == Some("top") {
                    if ch.motion.current_coord.row <= top + 1 && tick > 16 {
                        ch.motion.activate_path("home");
                    }
                }
            }

            term.step_all();

            let mut grid = vec![vec![(' ', None::<Color>); width]; height.max(term.canvas.height)];
            let gh = grid.len();
            let gw = width;
            for ch in term.get_characters() {
                if !ch.is_visible {
                    continue;
                }
                let x = ch.current_coord.column;
                let y = ch.current_coord.row;
                if x >= 0 && y >= 0 && (x as usize) < gw && (y as usize) < gh {
                    let col = ch
                        .animation
                        .current_character_visual
                        .colors
                        .and_then(|p| p.fg)
                        .or_else(|| {
                            ch.colors.and_then(|p| p.fg)
                        });
                    grid[y as usize][x as usize] = (ch.animation.current_character_visual.symbol, col);
                }
            }

            let mut out = String::new();
            for row in &grid {
                for (sym, col) in row {
                    if let Some(c) = col {
                        out.push_str(&format!(
                            "\x1b[38;2;{};{};{}m{}\x1b[0m",
                            c.r, c.g, c.b, sym
                        ));
                    } else if *sym != ' ' {
                        out.push_str("\x1b[38;2;180;180;180m");
                        out.push(*sym);
                        out.push_str("\x1b[0m");
                    } else {
                        out.push(' ');
                    }
                }
                out.push('\n');
            }
            frames.push(out);
        }

        frames
    }
}
