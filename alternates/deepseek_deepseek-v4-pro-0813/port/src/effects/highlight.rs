use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Highlight;

impl Highlight {
    pub fn new() -> Self {
        Highlight
    }
}

impl Effect for Highlight {
    fn name(&self) -> &str {
        "highlight"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = if input.is_empty() {
            vec![" "]
        } else {
            input
                .split('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .collect()
        };

        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut id = 1u32;

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let character = EffectCharacter::new(
                    id,
                    Coord::new(col as i32, row as i32),
                    ch,
                );
                terminal.add_character(character);
                id += 1;
            }
        }

        let total = terminal.get_characters().len();
        if total == 0 {
            return vec![terminal.render_frame()];
        }

        let total_f = total as f64;
        let sigma = (total_f / 10.0).max(1.0);
        let frame_count = (total * 2).clamp(40, 160);

        let base_fg = Color::WHITE;
        let base_bg = Color::BLACK;
        let highlight_fg = Color::BLACK;
        let highlight_bg = Color::new(255, 235, 59);

        let bg_gradient = Gradient::new(vec![(0.0, base_bg), (1.0, highlight_bg)]);
        let fg_gradient = Gradient::new(vec![(0.0, base_fg), (1.0, highlight_fg)]);

        let mut frames = Vec::with_capacity(frame_count);

        for frame_idx in 0..frame_count {
            let progress = frame_idx as f64 / (frame_count - 1) as f64;
            let center = progress * (total_f + 2.0 * sigma) - sigma;

            for (idx, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                let idx_f = idx as f64;
                let dist = (idx_f - center).abs();

                if dist <= sigma {
                    let raw = 1.0 - dist / sigma;
                    let smooth = raw * raw * (3.0 - 2.0 * raw);
                    character.color_pair = ColorPair::new(
                        fg_gradient.color_at(smooth),
                        bg_gradient.color_at(smooth),
                    );
                } else {
                    character.color_pair = ColorPair::new(base_fg, base_bg);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
