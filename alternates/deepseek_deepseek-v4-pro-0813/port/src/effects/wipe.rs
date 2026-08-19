use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};

pub struct Wipe;

impl Wipe {
    pub fn new() -> Self {
        Wipe
    }
}

impl Effect for Wipe {
    fn name(&self) -> &str {
        "wipe"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = if input.is_empty() {
            vec![""]
        } else {
            input.lines().collect()
        };

        let height = lines.len() as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as u16;

        let mut terminal = Terminal::new(width, height);

        // Add one character per input symbol.
        let mut id_counter = 0u32;
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let coord = Coord::new(col as i32, row as i32);
                let character = EffectCharacter::new(id_counter, coord, ch);
                terminal.add_character(character);
                id_counter += 1;
            }
        }

        // Initially hide all characters.
        for character in terminal.get_characters_mut() {
            character.visible = false;
        }

        let mut frames = Vec::new();

        // Initial blank frame (all hidden). It still carries ANSI SGR escapes
        // because the canvas renderer emits style codes for every cell.
        frames.push(terminal.render_frame());

        // Wipe from left to right by column. The leading column is highlighted.
        for col in 0..=width as i32 {
            for character in terminal.get_characters_mut() {
                if character.position.x < col {
                    character.visible = true;
                    character.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
                } else if character.position.x == col {
                    character.visible = true;
                    character.color_pair = ColorPair::new(Color::RED, Color::BLACK);
                } else {
                    character.visible = false;
                }
            }
            frames.push(terminal.render_frame());
        }

        frames
    }
}
