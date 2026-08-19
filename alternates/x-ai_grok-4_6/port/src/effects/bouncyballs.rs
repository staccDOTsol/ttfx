use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Bouncyballs;

impl Bouncyballs {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Bouncyballs {
    fn name(&self) -> &str {
        "bouncyballs"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let width = input
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(40)
            .max(1);
        let height = input.lines().count().max(1);
        let mut term = Terminal::from_input(input, width, height.max(8));

        let stops = vec![
            Color::from_hex("3A0088").unwrap_or(Color::rgb(58, 0, 136)),
            Color::from_hex("8800AA").unwrap_or(Color::rgb(136, 0, 170)),
            Color::from_hex("DD3355").unwrap_or(Color::rgb(221, 51, 85)),
            Color::from_hex("FFAA00").unwrap_or(Color::rgb(255, 170, 0)),
        ];
        let gradient = Gradient::new(stops, 24);
        let palette = gradient.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return vec!["\n".into()];
        }

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let color = palette[i % palette.len()];
                let start = Coord::new(ch.input_coord.column, -((i % 8) as i32) - 1);
                ch.motion.current_coord = start;
                ch.current_coord = start;
                let path = ch.motion.new_path("fall");
                path.speed = 0.35 + ((i % 5) as f64) * 0.08;
                path.easing = Easing::OutCubic;
                path.new_waypoint("home", ch.input_coord);
                ch.motion.activate_path("fall");

                let scn = ch.animation.new_scene("ball");
                scn.is_looping = true;
                scn.add_frame(
                    '●',
                    3,
                    Some(ColorPair {
                        fg: Some(color),
                        bg: None,
                    }),
                );
                scn.add_frame(
                    '◉',
                    3,
                    Some(ColorPair {
                        fg: Some(color),
                        bg: None,
                    }),
                );
                scn.add_frame(
                    ch.input_symbol,
                    4,
                    Some(ColorPair {
                        fg: Some(color),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("ball");
            }
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        let max_frames = 180usize;
        for tick in 0..max_frames {
            term.step_all();
            let settled = term.get_characters().iter().all(|c| {
                c.current_coord.column == c.input_coord.column
                    && c.current_coord.row == c.input_coord.row
            });
            frames.push(render_ansi(&term, width, height));
            if settled && tick > 12 {
                break;
            }
        }
        if frames.is_empty() {
            frames.push(render_ansi(&term, width, height));
        }
        frames
    }
}

fn render_ansi(term: &Terminal, width: usize, height: usize) -> String {
    let mut grid: Vec<Vec<(char, Option<Color>)>> =
        vec![vec![(' ', None); width]; height];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let col = ch.current_coord.column;
        let row = ch.current_coord.row;
        if col < 0 || row < 0 {
            continue;
        }
        let x = col as usize;
        let y = row as usize;
        if x < width && y < height {
            let vis = &ch.animation.current_character_visual;
            let fg = vis.colors.and_then(|p| p.fg).or_else(|| {
                ch.colors.and_then(|p| p.fg)
            });
            grid[y][x] = (vis.symbol, fg);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, fg) in row {
            if let Some(c) = fg {
                out.push_str(&format!(
                    "\x1b[38;2;{};{};{}m{}\x1b[0m",
                    c.r, c.g, c.b, sym
                ));
            } else {
                out.push(sym);
            }
        }
        out.push('\n');
    }
    out
}
