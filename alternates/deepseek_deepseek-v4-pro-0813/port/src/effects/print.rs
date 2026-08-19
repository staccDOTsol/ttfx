use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Print;

impl Print {
    pub fn new() -> Self {
        Print
    }
}

impl Effect for Print {
    fn name(&self) -> &str {
        "print"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Preserve text layout: split into rows and treat each character as a
        // terminal cell. Even a single trailing newline is respected by
        // splitting on '\n' rather than using input.lines().
        let rows: Vec<&str> = if input.is_empty() {
            vec![" "]
        } else {
            input.split('\n').collect()
        };

        let width = rows
            .iter()
            .map(|row| row.chars().count() as u16)
            .max()
            .unwrap_or(1)
            .max(1);
        let height = rows.len() as u16;

        let mut terminal = Terminal::new(width, height);

        let total_chars: usize = rows.iter().map(|row| row.chars().count()).sum();
        let gradient = Gradient::new(vec![
            (0.0, Color::new(0, 255, 255)),
            (0.5, Color::new(255, 0, 255)),
            (1.0, Color::new(255, 255, 255)),
        ]);

        let mut character_id = 0u32;
        let mut character_index = 0usize;

        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, symbol) in row.chars().enumerate() {
                let mut character = EffectCharacter::new(
                    character_id,
                    Coord::new(col_idx as i32, row_idx as i32),
                    symbol,
                );

                let progress = if total_chars <= 1 {
                    0.0
                } else {
                    character_index as f64 / (total_chars - 1) as f64
                };
                let fg = gradient.color_at(progress);
                character.color_pair = ColorPair::new(fg, Color::BLACK);

                terminal.add_character(character);

                character_id += 1;
                character_index += 1;
            }
        }

        vec![terminal.render_frame()]
    }
}
