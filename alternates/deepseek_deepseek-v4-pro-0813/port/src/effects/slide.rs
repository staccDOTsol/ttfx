use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::easing::{CubicOut, Easing};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

use super::Effect;

pub struct Slide;

impl Slide {
    pub fn new() -> Self {
        Slide
    }
}

impl Effect for Slide {
    fn name(&self) -> &str {
        "slide"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Determine canvas dimensions from the input text.
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len() as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as u16;

        if height == 0 || width == 0 {
            return Vec::new();
        }

        let mut terminal = Terminal::new(width, height);

        // Create a blue-to-cyan gradient for character foreground colors.
        let gradient = Gradient::new(vec![
            (0.0, Color::BLUE),
            (1.0, Color::new(0, 255, 255)), // cyan
        ]);

        // Collect final coordinates for each character in insertion order.
        let mut final_coords: Vec<Coord> = Vec::new();

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let final_coord = Coord::new(col as i32, row as i32);
                final_coords.push(final_coord);

                // Start offscreen to the left, keeping the same row.
                let start_coord = Coord::new(-(col as i32 + 1), row as i32);

                let id = terminal.characters.len() as u32;
                let character = EffectCharacter::new(id, start_coord, ch);

                // Apply gradient color, black background, bold, visible.
                let t = col as f64 / width.max(1) as f64;
                let fg = gradient.color_at(t);
                let bg = Color::BLACK;

                terminal.add_character(character);
                if let Some(c) = terminal.get_character_mut(id) {
                    c.color_pair = ColorPair::new(fg, bg);
                    c.bold = true;
                    c.visible = true;
                }
            }
        }

        let steps = 30;
        let mut frames = Vec::with_capacity(steps);
        let easing = CubicOut;

        for i in 0..steps {
            let t = i as f64 / (steps - 1) as f64;
            let eased_t = easing.ease(t);

            // Update every character's position.
            for (idx, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                let final_coord = final_coords[idx];
                let start_coord = Coord::new(-(final_coord.x as i32 + 1), final_coord.y);
                character.position = Coord::new(
                    start_coord.x + ((final_coord.x - start_coord.x) as f64 * eased_t).round() as i32,
                    start_coord.y + ((final_coord.y - start_coord.y) as f64 * eased_t).round() as i32,
                );
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
