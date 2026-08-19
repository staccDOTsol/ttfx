use super::Effect;
use crate::engine::{EffectCharacter, Terminal};
use crate::utils::easing::{Easing, SineInOut};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Expand;

impl Expand {
    pub fn new() -> Self {
        Expand
    }
}

impl Effect for Expand {
    fn name(&self) -> &str {
        "expand"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let input = if input.trim().is_empty() {
            "TerminalTextEffects"
        } else {
            input
        };

        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Vec::new();
        }

        let height = lines.len() as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let center = Coord::new((width / 2) as i32, (height / 2) as i32);

        let gradient = Gradient::new(vec![
            (0.0, Color::new(40, 160, 255)),
            (0.5, Color::new(160, 80, 255)),
            (1.0, Color::new(255, 120, 80)),
        ]);

        let max_dist = ((width as f64).powi(2) + (height as f64).powi(2)).sqrt() / 2.0;
        let mut final_coords = Vec::new();
        let mut id = 0u32;

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let distance = coord.distance(&center);
                let t = if max_dist > 0.0 {
                    (distance / max_dist).clamp(0.0, 1.0)
                } else {
                    0.0
                };

                let fg = gradient.color_at(t);
                let mut character = EffectCharacter::new(id, coord, ch);
                character.input_symbol = ch;
                character.color_pair = ColorPair::new(fg, Color::BLACK);
                character.visible = true;

                terminal.add_character(character);
                final_coords.push(coord);
                id += 1;
            }
        }

        let steps = 30u32;
        let mut frames = Vec::with_capacity((steps + 1) as usize);

        for step in 0..=steps {
            let t = step as f64 / steps as f64;
            let eased = SineInOut.ease(t);

            {
                let characters = terminal.get_characters_mut();
                for (i, final_coord) in final_coords.iter().enumerate() {
                    if i >= characters.len() {
                        break;
                    }

                    let x = center.x as f64 + (final_coord.x - center.x) as f64 * eased;
                    let y = center.y as f64 + (final_coord.y - center.y) as f64 * eased;
                    characters[i].position = Coord::new(x.round() as i32, y.round() as i32);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
