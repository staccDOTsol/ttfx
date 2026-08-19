use std::collections::HashMap;

use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

fn lerp_color(start: Color, end: Color, t: f64) -> Color {
    let r = (start.r as f64 + (end.r as f64 - start.r as f64) * t).round() as u8;
    let g = (start.g as f64 + (end.g as f64 - start.g as f64) * t).round() as u8;
    let b = (start.b as f64 + (end.b as f64 - start.b as f64) * t).round() as u8;
    Color::new(r, g, b)
}

pub struct Smoke;

impl Smoke {
    pub fn new() -> Self {
        Smoke
    }
}

impl Effect for Smoke {
    fn name(&self) -> &str {
        "smoke"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let base_width = lines.iter().map(|line| line.chars().count()).max().unwrap_or(1);
        let base_height = lines.len().max(1);

        let width = (base_width + 4).max(1) as u16;
        let height = (base_height + 18) as u16;

        let mut terminal = Terminal::new(width, height);

        let non_space_count = input
            .chars()
            .filter(|c| *c != '\n' && *c != '\r' && *c != ' ')
            .count();

        let color_gradient = Gradient::new(vec![
            (0.0, Color::new(70, 70, 70)),
            (0.5, Color::new(160, 160, 160)),
            (1.0, Color::new(235, 235, 235)),
        ]);

        let mut id: u32 = 0;
        let mut initial_positions: HashMap<u32, Coord> = HashMap::new();
        let mut initial_colors: HashMap<u32, Color> = HashMap::new();

        let start_y = (height as i32 - base_height as i32 - 2).max(0);
        let mut processed = 0;

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }

                let t = processed as f64 / non_space_count.max(1) as f64;
                let fg = color_gradient.color_at(t);
                let coord = Coord::new(2 + col as i32, start_y + row as i32);

                let mut character = EffectCharacter::new(id, coord, ch);
                character.color_pair = ColorPair::new(fg, Color::new(12, 12, 12));

                initial_positions.insert(id, coord);
                initial_colors.insert(id, fg);
                terminal.add_character(character);

                id += 1;
                processed += 1;
            }
        }

        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        let total_steps = 26;
        for step in 1..=total_steps {
            let progress = step as f64 / total_steps as f64;
            let rise = progress * height as f64;

            for character in terminal.get_characters_mut() {
                if !character.visible {
                    continue;
                }

                let initial = initial_positions[&character.id];
                let drift = (progress * 8.0 * (character.id as f64 * 0.27).sin()) as i32;
                character.position = Coord::new(initial.x + drift, initial.y - rise as i32);

                let start_fg = initial_colors[&character.id];
                let end_fg = Color::new(10, 10, 10);
                character.color_pair = ColorPair::new(
                    lerp_color(start_fg, end_fg, progress),
                    Color::new(10, 10, 10),
                );

                if progress > 0.55 {
                    let smoke_symbols = ['░', '▒', '▓'];
                    let idx = ((progress - 0.55) * 8.0) as usize;
                    let idx = idx.min(smoke_symbols.len() - 1);
                    character.symbol = smoke_symbols[idx];
                    character.dim = true;
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
