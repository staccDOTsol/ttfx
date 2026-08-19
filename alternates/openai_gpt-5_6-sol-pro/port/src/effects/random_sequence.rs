use super::Effect;
use crate::engine::{CharacterId, EffectCharacter, Terminal};
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

pub struct RandomSequence;

impl RandomSequence {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RandomSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for RandomSequence {
    fn name(&self) -> &str {
        "random_sequence"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let gradient = Gradient::new(
            [
                Color::new(138, 0, 138),
                Color::new(0, 209, 255),
                Color::new(255, 255, 255),
            ],
            (width + height).max(2),
        );
        let colors = gradient.colors();

        let mut terminal = Terminal::new(width, height);
        let mut character_indices = Vec::new();
        let diagonal_span = width.saturating_sub(1) + height.saturating_sub(1);

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let diagonal = column + row;
                let color_index = if diagonal_span == 0 {
                    colors.len() - 1
                } else {
                    diagonal * (colors.len() - 1) / diagonal_span
                };

                let mut character = EffectCharacter::new(
                    CharacterId(character_indices.len() as u32),
                    symbol.to_string(),
                    Coord::new(column as i32, row as i32),
                );
                character.visible = false;
                character.style = Style::with_colors(ColorPair::new(
                    Some(colors[color_index]),
                    None,
                ));

                terminal.add_character(character);
                character_indices.push(character_indices.len());
            }
        }

        if character_indices.is_empty() {
            terminal.canvas.set(
                Coord::new(0, 0),
                " ",
                Style::with_colors(ColorPair::new(
                    Some(Color::new(0, 209, 255)),
                    None,
                )),
            );
            return vec![terminal.canvas.render()];
        }

        let mut state = input.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
            hash.wrapping_mul(0x100000001b3) ^ u64::from(byte)
        });
        state ^= character_indices.len() as u64;

        for index in (1..character_indices.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let swap_index = (state as usize) % (index + 1);
            character_indices.swap(index, swap_index);
        }

        let reveal_count = ((character_indices.len() as f64 * 0.004) as usize).max(1);
        let mut frames = Vec::new();

        for batch in character_indices.chunks(reveal_count) {
            for &index in batch {
                terminal.characters_mut()[index].visible = true;
            }
            frames.push(terminal.step_frame());
        }

        frames
    }
}
