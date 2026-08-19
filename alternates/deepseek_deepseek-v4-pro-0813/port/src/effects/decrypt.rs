use super::Effect;
use crate::engine::{EffectCharacter, Terminal};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const LEAD_IN: usize = 8;
const LEAD_OUT: usize = 12;
const ENCRYPTED_SYMBOLS: [char; 16] = [
    '!', '@', '#', '$', '%', '&', '?', '*', '+', '=', '~', '^', '/', '|', '<', '>',
];

pub struct Decrypt;

impl Decrypt {
    pub fn new() -> Self {
        Decrypt
    }
}

impl Effect for Decrypt {
    fn name(&self) -> &str {
        "decrypt"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let input = if input.is_empty() { " " } else { input };
        let lines: Vec<&str> = input.lines().collect();
        let width = lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1)
            .min(u16::MAX as usize) as u16;
        let height = (lines.len().min(u16::MAX as usize) as u16).max(1);

        let mut terminal = Terminal::new(width, height);

        let gradient = Gradient::new(vec![
            (0.0, Color::new(0, 90, 0)),
            (0.5, Color::new(0, 220, 0)),
            (1.0, Color::new(160, 255, 160)),
        ]);

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let idx = row * width as usize + col;
                let mut character = EffectCharacter::new(
                    idx as u32,
                    Coord::new(col as i32, row as i32),
                    ENCRYPTED_SYMBOLS[idx % ENCRYPTED_SYMBOLS.len()],
                );
                character.input_symbol = ch;
                let t = col as f64 / width.max(1) as f64;
                character.color_pair = ColorPair::new(gradient.color_at(t), Color::BLACK);
                terminal.add_character(character);
            }
        }

        let total_chars = terminal.get_characters().len();
        let total_frames = total_chars + LEAD_IN + LEAD_OUT;
        let mut frames = Vec::with_capacity(total_frames);

        for frame_idx in 0..total_frames {
            let reveal_front = if frame_idx < LEAD_IN {
                None
            } else {
                Some(frame_idx - LEAD_IN)
            };

            {
                let characters = terminal.get_characters_mut();
                for (i, character) in characters.iter_mut().enumerate() {
                    match reveal_front {
                        None => {
                            // Pre-reveal scramble: everything cycles through encrypted symbols.
                            character.symbol =
                                ENCRYPTED_SYMBOLS[(i + frame_idx) % ENCRYPTED_SYMBOLS.len()];
                            character.color_pair =
                                ColorPair::new(Color::new(0, 90, 0), Color::BLACK);
                        }
                        Some(front) => {
                            if i < front {
                                character.symbol = character.input_symbol;
                                let t = i as f64 / (total_chars.max(1)) as f64;
                                character.color_pair =
                                    ColorPair::new(gradient.color_at(t), Color::BLACK);
                            } else if i == front {
                                // Highlight the character currently being decrypted.
                                character.symbol = '*';
                                character.color_pair =
                                    ColorPair::new(Color::WHITE, Color::BLACK);
                            } else {
                                // Still obscured by a cycling encrypted symbol.
                                character.symbol =
                                    ENCRYPTED_SYMBOLS[(i + frame_idx) % ENCRYPTED_SYMBOLS.len()];
                                character.color_pair =
                                    ColorPair::new(Color::new(0, 90, 0), Color::BLACK);
                            }
                        }
                    }
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
