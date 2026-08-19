use super::Effect;
use crate::engine::{EffectCharacter, Terminal};
use crate::utils::easing::{CubicOut, Easing};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const START_COLOR: Color = Color { r: 0, g: 180, b: 255 };

const FINAL_COLORS: [Color; 6] = [
    Color::WHITE,
    Color { r: 255, g: 255, b: 180 },
    Color { r: 150, g: 255, b: 255 },
    Color { r: 255, g: 180, b: 255 },
    Color { r: 180, g: 255, b: 180 },
    Color { r: 255, g: 220, b: 180 },
];

pub struct Pour;

impl Pour {
    pub fn new() -> Self {
        Pour
    }
}

impl Effect for Pour {
    fn name(&self) -> &str {
        "pour"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let input = if input.trim().is_empty() {
            "TerminalTextEffects"
        } else {
            input
        };

        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut final_positions: Vec<Coord> = Vec::new();
        let mut next_id: u32 = 0;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                if symbol == ' ' {
                    continue;
                }

                let final_coord = Coord::new(column as i32, row as i32);
                let mut character = EffectCharacter::new(next_id, final_coord, symbol);

                // Start well above the canvas so characters appear to pour in
                // from the top edge before settling on their input coordinates.
                character.position = Coord::new(column as i32, -(height as i32));
                character.color_pair = ColorPair::new(START_COLOR, Color::BLACK);

                terminal.add_character(character);
                final_positions.push(final_coord);
                next_id += 1;
            }
        }

        if terminal.get_characters().is_empty() {
            return vec![terminal.render_frame()];
        }

        let mut durations = vec![0usize; terminal.get_characters().len()];
        for (index, _character) in terminal.get_characters().iter().enumerate() {
            let speed = 0.55 + (index % 7) as f64 * 0.08;
            let start_y = -(height as f64);
            let final_y = final_positions[index].y as f64;
            let distance = (final_y - start_y).abs();

            durations[index] = (distance / speed * 0.55).round() as usize + 8;
        }

        let total_frames = durations.iter().copied().max().unwrap_or(1).max(1);
        let mut frames = Vec::with_capacity(total_frames);
        let cubic_out = CubicOut;

        for frame_index in 0..total_frames {
            for (character_index, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                let duration = durations[character_index] as f64;
                let t = (frame_index as f64 / duration).min(1.0);
                let eased = cubic_out.ease(t);

                let final_coord = final_positions[character_index];
                let start_y = -(height as f64);
                let current_y = start_y + (final_coord.y as f64 - start_y) * eased;

                character.position = Coord::new(final_coord.x, current_y.round() as i32);

                let final_color = FINAL_COLORS[character_index % FINAL_COLORS.len()];
                let gradient = Gradient::new(vec![(0.0, START_COLOR), (1.0, final_color)]);
                character.color_pair = ColorPair::new(gradient.color_at(eased), Color::BLACK);
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
