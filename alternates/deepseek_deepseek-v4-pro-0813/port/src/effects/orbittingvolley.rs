use super::Effect;
use crate::engine::{EffectCharacter, Terminal};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use std::f64::consts::PI;

pub struct Orbittingvolley;

impl Orbittingvolley {
    pub fn new() -> Self {
        Orbittingvolley
    }
}

impl Effect for Orbittingvolley {
    fn name(&self) -> &str {
        "orbittingvolley"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Keep only printable characters (ignore newlines and carriage returns).
        let input_chars: Vec<char> = input
            .chars()
            .filter(|c| *c != '\n' && *c != '\r')
            .collect();

        if input_chars.is_empty() {
            return Vec::new();
        }

        // Fixed canvas width, height scales with the number of characters.
        // At least 3 rows so there is vertical room to orbit.
        let cols: usize = 40;
        let rows = ((input_chars.len() + cols - 1) / cols).max(3);
        let width = cols as u16;
        let height = rows as u16;

        // Store the original input coordinate for each character.
        let input_coords: Vec<Coord> = input_chars
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let col = (i % cols) as i32;
                let row = (i / cols) as i32;
                Coord::new(col, row)
            })
            .collect();

        let mut terminal = Terminal::new(width, height);

        // Give every character a visible style so SGR codes are guaranteed.
        for (i, &ch) in input_chars.iter().enumerate() {
            let mut character = EffectCharacter::new(i as u32, input_coords[i], ch);
            character.bold = true; // additional SGR attribute
            terminal.add_character(character);
        }

        // Simple orange-to-blue gradient for animated color.
        let start_color = Color::new(255, 120, 0);
        let end_color = Color::new(0, 120, 255);
        let gradient = Gradient::new(vec![(0.0, start_color), (1.0, end_color)]);

        let total_frames = 40;
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 / 2.0;
        let mut frames = Vec::with_capacity(total_frames + 1);

        // Rotate all characters around the canvas centre.
        for frame in 0..total_frames {
            let t = if total_frames > 1 {
                frame as f64 / (total_frames - 1) as f64
            } else {
                0.0
            };
            let color = gradient.color_at(t);
            let color_pair = ColorPair::new(color, Color::BLACK);

            let angle = 2.0 * PI * (frame as f64) / (total_frames as f64);
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            for (idx, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                let orig = input_coords[idx];
                let dx = orig.x as f64 - center_x;
                let dy = orig.y as f64 - center_y;

                let rot_x = dx * cos_a - dy * sin_a;
                let rot_y = dx * sin_a + dy * cos_a;

                let new_x = (center_x + rot_x).round() as i32;
                let new_y = (center_y + rot_y).round() as i32;

                // Clamp to canvas boundaries to avoid out-of-bounds coordinates.
                character.position = Coord::new(
                    new_x.clamp(0, (width - 1) as i32),
                    new_y.clamp(0, (height - 1) as i32),
                );
                character.color_pair = color_pair;
            }

            frames.push(terminal.render_frame());
        }

        // Final frame: characters return to their exact input positions,
        // using the final gradient color.
        let final_color = gradient.color_at(1.0);
        let final_pair = ColorPair::new(final_color, Color::BLACK);
        for (idx, character) in terminal.get_characters_mut().iter_mut().enumerate() {
            character.position = input_coords[idx];
            character.color_pair = final_pair;
        }
        frames.push(terminal.render_frame());

        frames
    }
}
