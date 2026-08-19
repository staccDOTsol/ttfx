use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Synthgrid;

impl Synthgrid {
    pub fn new() -> Self {
        Self
    }
}

fn sgr(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

fn sgr_bg(c: Color) -> String {
    format!("\x1b[48;2;{};{};{}m", c.r, c.g, c.b)
}

const RESET: &str = "\x1b[0m";

impl Effect for Synthgrid {
    fn name(&self) -> &str {
        "synthgrid"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let grid_stops = vec![
            Color::from_hex("f72585").unwrap_or(Color::rgb(247, 37, 133)),
            Color::from_hex("7209b7").unwrap_or(Color::rgb(114, 9, 183)),
            Color::from_hex("3a0ca3").unwrap_or(Color::rgb(58, 12, 163)),
            Color::from_hex("4361ee").unwrap_or(Color::rgb(67, 97, 238)),
            Color::from_hex("4cc9f0").unwrap_or(Color::rgb(76, 201, 240)),
        ];
        let grid_grad = Gradient::new(grid_stops, (width + height).max(8));
        let grid_colors = grid_grad.colors();

        let text_stops = vec![
            Color::from_hex("ff006e").unwrap_or(Color::rgb(255, 0, 110)),
            Color::from_hex("8338ec").unwrap_or(Color::rgb(131, 56, 236)),
            Color::from_hex("3a86ff").unwrap_or(Color::rgb(58, 134, 255)),
        ];
        let text_grad = Gradient::new(text_stops, width.max(4));
        let text_colors = text_grad.colors();

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, false);
        }

        let mut frames = Vec::new();
        let w = width as i32;
        let h = height as i32;

        // Sweep horizontal grid lines
        for row in 0..h {
            let color = grid_colors[(row as usize) % grid_colors.len().max(1)];
            let mut line = String::new();
            for col in 0..w {
                line.push_str(&sgr(color));
                line.push_str(&sgr_bg(Color::rgb(
                    color.r / 6,
                    color.g / 6,
                    color.b / 6,
                )));
                line.push(if col % 4 == 0 { '│' } else { '─' });
                line.push_str(RESET);
            }
            line.push('\n');
            let mut frame = String::new();
            for r in 0..h {
                if r == row {
                    frame.push_str(&line);
                } else {
                    frame.push_str(&" ".repeat(width));
                    frame.push('\n');
                }
            }
            frames.push(frame);
        }

        // Sweep vertical grid
        for col in 0..w {
            let color = grid_colors[(col as usize) % grid_colors.len().max(1)];
            let mut frame = String::new();
            for row in 0..h {
                for c in 0..w {
                    if c == col {
                        frame.push_str(&sgr(color));
                        frame.push('│');
                        frame.push_str(RESET);
                    } else if row % 3 == 0 {
                        let gc = grid_colors[(c as usize) % grid_colors.len().max(1)];
                        frame.push_str(&sgr(gc));
                        frame.push('·');
                        frame.push_str(RESET);
                    } else {
                        frame.push(' ');
                    }
                }
                frame.push('\n');
            }
            frames.push(frame);
        }

        // Reveal input characters along a center-out synth grid
        for id in &ids {
            term.set_character_visibility(*id, true);
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let idx = (ch.input_coord.column.max(0) as usize) % text_colors.len().max(1);
                let fg = text_colors[idx];
                let scn = ch.animation.new_scene("synth");
                scn.add_frame(
                    ch.input_symbol,
                    2,
                    Some(ColorPair {
                        fg: Some(fg),
                        bg: None,
                    }),
                );
                ch.animation.activate_scene("synth");
                ch.animation.set_appearance(ch.input_symbol, Some(fg));
            }
        }

        let reveal_steps = (width.max(height) + 4).min(48);
        for step in 0..reveal_steps {
            term.step_all();
            let mut frame = String::new();
            for row in 0..h {
                for col in 0..w {
                    let coord = Coord::new(col, row);
                    let mut painted = false;
                    for ch in term.get_characters() {
                        if ch.is_visible && ch.current_coord == coord {
                            let idx = (col as usize + step) % text_colors.len().max(1);
                            let fg = text_colors[idx];
                            let pulse = grid_colors[step % grid_colors.len().max(1)];
                            frame.push_str(&sgr(fg));
                            if step < 6 {
                                frame.push_str(&sgr_bg(Color::rgb(
                                    pulse.r / 5,
                                    pulse.g / 5,
                                    pulse.b / 5,
                                )));
                            }
                            frame.push(ch.animation.current_character_visual.symbol);
                            frame.push_str(RESET);
                            painted = true;
                            break;
                        }
                    }
                    if !painted {
                        if row % 3 == 0 || col % 4 == 0 {
                            let gc = grid_colors
                                [((row + col + step as i32) as usize) % grid_colors.len().max(1)];
                            frame.push_str(&sgr(Color::rgb(gc.r / 3, gc.g / 3, gc.b / 3)));
                            frame.push(if row % 3 == 0 { '─' } else { '│' });
                            frame.push_str(RESET);
                        } else {
                            frame.push(' ');
                        }
                    }
                }
                frame.push('\n');
            }
            frames.push(frame);
        }

        // Final hold: fully styled input
        for _ in 0..4 {
            let mut frame = String::new();
            for row in 0..h {
                for col in 0..w {
                    let coord = Coord::new(col, row);
                    let mut painted = false;
                    for ch in term.get_characters() {
                        if ch.input_coord == coord {
                            let idx = (col as usize) % text_colors.len().max(1);
                            frame.push_str(&sgr(text_colors[idx]));
                            frame.push(ch.input_symbol);
                            frame.push_str(RESET);
                            painted = true;
                            break;
                        }
                    }
                    if !painted {
                        frame.push(' ');
                    }
                }
                frame.push('\n');
            }
            frames.push(frame);
        }

        frames
    }
}
