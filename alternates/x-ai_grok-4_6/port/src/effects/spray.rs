use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Spray;

impl Spray {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Spray {
    fn default() -> Self {
        Self::new()
    }
}

fn sgr_fg(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

const RESET: &str = "\x1b[0m";

impl Effect for Spray {
    fn name(&self) -> &str {
        "spray"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let stops = vec![
            Color::rgb(0x00, 0xbf, 0xff),
            Color::rgb(0xff, 0x8c, 0x00),
            Color::rgb(0xff, 0x00, 0x7f),
            Color::rgb(0x7c, 0xfc, 0x00),
        ];
        let gradient = Gradient::new(stops, 24);
        let spectrum = gradient.colors();
        if spectrum.is_empty() {
            return vec![input.to_string()];
        }

        let origin = Coord::new(width as i32 / 2, -2);
        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return vec![format!("{RESET}")];
        }

        for (i, id) in ids.iter().enumerate() {
            let color = spectrum[i % spectrum.len()];
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let dest = ch.input_coord;
                ch.motion.current_coord = origin;
                ch.current_coord = origin;
                ch.colors = Some(ColorPair {
                    fg: Some(color),
                    bg: None,
                });
                let path = ch.motion.new_path("spray");
                path.speed = 0.08 + ((i as f64 * 0.013) % 0.12);
                path.easing = Easing::OutQuad;
                path.new_waypoint("home", dest);
                let scn = ch.animation.new_scene("spray");
                scn.add_frame(ch.input_symbol, 1, Some(ColorPair {
                    fg: Some(color),
                    bg: None,
                }));
                ch.animation.activate_scene("spray");
                ch.animation.set_appearance(ch.input_symbol, Some(color));
            }
            term.set_character_visibility(*id, true);
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                ch.motion.activate_path("spray");
            }
        }

        let mut frames = Vec::new();
        for _ in 0..90 {
            term.step_all();
            let mut grid = vec![vec![' '; width]; height];
            let mut color_grid: Vec<Vec<Option<Color>>> = vec![vec![None; width]; height];
            for ch in term.get_characters() {
                if !ch.is_visible {
                    continue;
                }
                let x = ch.current_coord.column;
                let y = ch.current_coord.row;
                if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                    grid[y as usize][x as usize] = ch.animation.current_character_visual.symbol;
                    color_grid[y as usize][x as usize] = ch
                        .colors
                        .and_then(|p| p.fg)
                        .or_else(|| {
                            ch.animation
                                .current_character_visual
                                .colors
                                .and_then(|p| p.fg)
                        });
                }
            }
            let mut out = String::new();
            for y in 0..height {
                for x in 0..width {
                    let sym = grid[y][x];
                    if let Some(c) = color_grid[y][x] {
                        out.push_str(&sgr_fg(c));
                        out.push(sym);
                        out.push_str(RESET);
                    } else {
                        out.push(sym);
                    }
                }
                out.push('\n');
            }
            frames.push(out);
        }
        frames
    }
}
