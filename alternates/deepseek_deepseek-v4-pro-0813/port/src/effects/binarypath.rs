use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};

pub struct Binarypath;

impl Binarypath {
    pub fn new() -> Self {
        Binarypath
    }
}

impl Effect for Binarypath {
    fn name(&self) -> &str {
        "binarypath"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut input_coords = Vec::new();
        let mut input_symbols = Vec::new();

        let mut character_id = 0u32;
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let mut character = EffectCharacter::new(character_id, coord, ch);
                // Give each character a green/cyan binary-path colour.
                let g = 160 + (character_id * 17 % 96) as u8;
                character.color_pair = ColorPair::new(
                    Color::new(80, g, 140 + (character_id * 11 % 100) as u8),
                    Color::BLACK,
                );
                terminal.add_character(character);
                input_coords.push(coord);
                input_symbols.push(ch);
                character_id += 1;
            }
        }

        // If the input was empty, add a styled space so the frame still carries ANSI colours.
        if terminal.characters.is_empty() {
            let mut placeholder = EffectCharacter::new(0, Coord::new(0, 0), ' ');
            placeholder.color_pair = ColorPair::new(Color::new(0, 255, 128), Color::BLACK);
            terminal.add_character(placeholder);
            input_coords.push(Coord::new(0, 0));
            input_symbols.push(' ');
        }

        let start_positions = compute_binary_tree_positions(
            terminal.characters.len(),
            width,
            height,
        );

        for (i, character) in terminal.characters.iter_mut().enumerate() {
            character.position = start_positions[i];
        }

        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        let total_steps = 30usize;
        for step in 1..=total_steps {
            let linear_t = step as f64 / total_steps as f64;
            // Smoothstep gives a natural ease without requiring the easing module.
            let t = linear_t * linear_t * (3.0 - 2.0 * linear_t);

            for (i, character) in terminal.characters.iter_mut().enumerate() {
                let start = start_positions[i];
                let end = input_coords[i];

                let x = (start.x as f64 + (end.x as f64 - start.x as f64) * t)
                    .round()
                    .clamp(0.0, width as f64 - 1.0) as i32;
                let y = (start.y as f64 + (end.y as f64 - start.y as f64) * t)
                    .round()
                    .clamp(0.0, height as f64 - 1.0) as i32;

                character.position = Coord::new(x, y);
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}

fn compute_binary_tree_positions(count: usize, width: u16, height: u16) -> Vec<Coord> {
    if count == 0 {
        return Vec::new();
    }

    let depth = if count <= 1 {
        0
    } else {
        (count as f64).log2().ceil() as i32
    };

    let mut positions = Vec::with_capacity(count);
    let root_x = width as f64 / 2.0;
    let dy = height as f64 / (depth as f64 + 1.0);
    let initial_step = width as f64 / 4.0;

    for i in 0..count {
        let mut x = root_x;
        let mut y = 0.0;
        let mut step = initial_step;

        for level in 0..depth {
            let bit = (i >> (depth - 1 - level)) & 1;
            if bit == 0 {
                x -= step;
            } else {
                x += step;
            }
            y += dy;
            step *= 0.5;
        }

        let col = x.round().clamp(0.0, width as f64 - 1.0) as i32;
        let row = y.round().clamp(0.0, height as f64 - 1.0) as i32;
        positions.push(Coord::new(col, row));
    }

    positions
}
