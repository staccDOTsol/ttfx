use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Overflow;

impl Overflow {
    pub fn new() -> Self {
        Self
    }
}

fn sgr(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

fn reset() -> &'static str {
    "\x1b[0m"
}

impl Effect for Overflow {
    fn name(&self) -> &str {
        "overflow"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = if input.is_empty() {
            vec![""]
        } else {
            input.lines().collect()
        };
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);

        let mut term = Terminal::from_input(input, width, height);
        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        let gradient = Gradient::new(
            vec![
                Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
                Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
                Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
            ],
            12,
        );
        let palette = gradient.colors();
        if palette.is_empty() {
            return vec![input.to_string()];
        }

        // Paint initial scenes so engine visuals carry ColorPairs.
        {
            let chars = term.get_characters_mut();
            for ch in chars.iter_mut() {
                let idx = ((ch.input_coord.column + ch.input_coord.row).unsigned_abs() as usize)
                    % palette.len();
                let scn = ch.animation.new_scene("overflow");
                scn.add_frame(
                    ch.input_symbol,
                    2,
                    Some(ColorPair {
                        fg: Some(palette[idx]),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("overflow");
                let path = ch.motion.new_path("drop");
                path.speed = 0.35;
                path.new_waypoint(
                    "bottom",
                    Coord::new(ch.input_coord.column, ch.input_coord.row + height as i32),
                );
            }
        }

        let mut out = Vec::new();
        let cycles = 3usize;
        let steps_per = (height as i32 + 4).max(8) as usize;

        for cycle in 0..cycles {
            // Reset coords to input, activate drop
            {
                let chars = term.get_characters_mut();
                for ch in chars.iter_mut() {
                    ch.current_coord = ch.input_coord;
                    ch.motion.current_coord = ch.input_coord;
                    ch.motion.activate_path("drop");
                    let idx = ((ch.input_coord.column + ch.input_coord.row + cycle as i32)
                        .unsigned_abs() as usize)
                        % palette.len();
                    if let Some(scn) = ch.animation.query_scene("overflow") {
                        let _ = scn;
                    }
                    ch.animation.set_appearance(ch.input_symbol, Some(palette[idx]));
                    ch.animation.activate_scene("overflow");
                }
            }

            for step in 0..steps_per {
                term.step_all();
                // Build a styled frame: wrap each visible glyph with SGR from gradient
                let mut grid: Vec<Vec<Option<(char, Color)>>> =
                    vec![vec![None; width]; height];
                for ch in term.get_characters() {
                    if !ch.is_visible {
                        continue;
                    }
                    let col = ch.current_coord.column;
                    let row = ch.current_coord.row;
                    if col < 0 || row < 0 {
                        continue;
                    }
                    let c = col as usize;
                    let r = row as usize;
                    if c >= width || r >= height {
                        continue;
                    }
                    let idx = ((ch.input_coord.column
                        + ch.input_coord.row
                        + cycle as i32
                        + step as i32)
                        .unsigned_abs() as usize)
                        % palette.len();
                    let color = ch
                        .animation
                        .current_character_visual
                        .colors
                        .and_then(|p| p.fg)
                        .unwrap_or(palette[idx]);
                    grid[r][c] = Some((ch.animation.current_character_visual.symbol, color));
                }
                let mut frame = String::new();
                for r in 0..height {
                    for c in 0..width {
                        match grid[r][c] {
                            Some((sym, color)) => {
                                frame.push_str(&sgr(color));
                                frame.push(sym);
                                frame.push_str(reset());
                            }
                            None => frame.push(' '),
                        }
                    }
                    frame.push('\n');
                }
                out.push(frame);
            }
        }

        // Final settle: characters back at input coords, last gradient color
        {
            let last = *palette.last().unwrap();
            let chars = term.get_characters_mut();
            for ch in chars.iter_mut() {
                ch.current_coord = ch.input_coord;
                ch.motion.current_coord = ch.input_coord;
                ch.animation.set_appearance(ch.input_symbol, Some(last));
            }
        }
        let mut grid: Vec<Vec<Option<(char, Color)>>> = vec![vec![None; width]; height];
        let last = *palette.last().unwrap();
        for ch in term.get_characters() {
            let c = ch.input_coord.column as usize;
            let r = ch.input_coord.row as usize;
            if c < width && r < height {
                grid[r][c] = Some((ch.input_symbol, last));
            }
        }
        let mut frame = String::new();
        for r in 0..height {
            for c in 0..width {
                match grid[r][c] {
                    Some((sym, color)) => {
                        frame.push_str(&sgr(color));
                        frame.push(sym);
                        frame.push_str(reset());
                    }
                    None => frame.push(' '),
                }
            }
            frame.push('\n');
        }
        out.push(frame);
        out
    }
}
