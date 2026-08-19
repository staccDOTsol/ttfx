use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use super::Effect;
use std::f64::consts::PI;

/// The `bubbles` effect: characters form small clusters that float upward,
/// wobble, and change color before popping back to their original positions.
pub struct Bubbles;

impl Bubbles {
    pub fn new() -> Self {
        Bubbles
    }
}

impl Effect for Bubbles {
    fn name(&self) -> &str {
        "bubbles"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Fixed terminal size. For most inputs this is plenty of room.
        let width: u16 = 80;
        let height: u16 = 24;
        let mut terminal = Terminal::new(width, height);

        // Collect input characters, preserving spaces and newlines as spaces.
        let chars: Vec<char> = input.chars().collect();
        if chars.is_empty() {
            return Vec::new();
        }

        // Place characters in a block starting at (1,1).
        // We wrap to the next row when reaching the right edge.
        let mut col: i32 = 1;
        let mut row: i32 = 1;
        for (i, ch) in chars.iter().enumerate() {
            let id = i as u32;
            let coord = Coord::new(col, row);
            let mut character = EffectCharacter::new(id, coord, *ch);
            // Initial color based on index (creates a general gradient from blue to cyan).
            character.color_pair = ColorPair::new(
                Color::new(50, 150, 200),
                Color::BLACK,
            );
            terminal.add_character(character);

            // Advance position
            col += 1;
            if col >= width as i32 - 1 {
                col = 1;
                row += 1;
                if row >= height as i32 - 1 {
                    break; // stop adding if we exceed the canvas
                }
            }
        }

        // A simple gradient used for dynamic color changes.
        let gradient = Gradient::new(vec![
            (0.0, Color::new(0, 100, 200)),
            (0.5, Color::new(0, 200, 255)),
            (1.0, Color::new(255, 255, 255)),
        ]);

        let num_frames = 120;
        let mut frames = Vec::with_capacity(num_frames);

        // Phase offsets per character for independent motion.
        // We'll store the base coordinates (original positions) separately.
        let base_positions: Vec<Coord> = terminal
            .get_characters()
            .iter()
            .map(|c| c.position)
            .collect();

        for frame_idx in 0..num_frames {
            let t = frame_idx as f64 / num_frames as f64;

            // Update each character's position and color.
            for (i, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                let base = base_positions[i];
                let phase = (i as f64) * PI / 6.0; // vary per character

                // Bubble‑like motion: circular wobble plus upward drift.
                // The amplitude grows, then shrinks near the end (simulating pop).
                let amp = 4.0 * (1.0 - t).powi(2) + 0.5;
                let idx_float = i as f64;
                let angle = t * 4.0 * PI + phase;
                let dx = amp * angle.cos();
                let dy = amp * angle.sin() - t * 12.0; // drift upward

                let new_x = base.x as f64 + dx;
                let new_y = base.y as f64 + dy;

                // Keep within canvas bounds to avoid completely disappearing.
                let clamped_x = new_x.max(0.0).min(width as f64 - 1.0);
                let clamped_y = new_y.max(0.0).min(height as f64 - 1.0);
                character.position = Coord::new(clamped_x.round() as i32, clamped_y.round() as i32);

                // Update color using the gradient.
                let color_t = (t + phase / (2.0 * PI)) % 1.0;
                let fg = gradient.color_at(color_t);
                character.color_pair = ColorPair::new(fg, Color::BLACK);
            }

            // Render the current state.
            let frame_str = terminal.render_frame();
            frames.push(frame_str);
        }

        frames
    }
}
