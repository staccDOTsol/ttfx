use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::easing::{CubicOut, Easing};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Sweep {
    // No configuration fields
}

impl Sweep {
    pub fn new() -> Self {
        Sweep {}
    }
}

impl Effect for Sweep {
    fn name(&self) -> &str {
        "sweep"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Parse input lines
        let lines: Vec<&str> = if input.is_empty() {
            vec!["Sweep"] // fallback
        } else {
            input.lines().collect()
        };

        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut final_coords: Vec<Coord> = Vec::new();

        // Add non-space characters, initially placed off-screen.
        let mut id = 0u32;
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let coord = Coord::new(x as i32, y as i32);
                    let mut character = EffectCharacter::new(id, Coord::new(-1, y as i32), ch);
                    // Default colors: will be updated during animation
                    character.color_pair = ColorPair::new(Color::new(0, 255, 255), Color::BLACK);
                    character.visible = false;
                    terminal.add_character(character);
                    final_coords.push(coord);
                    id += 1;
                }
            }
        }

        // If no non-space characters, return a single frame (empty canvas with spaces).
        if terminal.characters.is_empty() {
            return vec![terminal.render_frame()];
        }

        // Animation parameters
        let frames_per_column = 2; // frames delay per column
        let move_duration = 8; // frames for a character to travel from start to final
        let total_frames = (width as usize * frames_per_column) + move_duration + 5;

        // Colors
        let sweep_color = Color::new(0, 255, 255); // cyan
        let final_color = Color::WHITE;
        let bg_color = Color::BLACK;
        let gradient = Gradient::new(vec![(0.0, sweep_color), (1.0, final_color)]);

        // Easing function for movement
        let cubic_out = CubicOut;

        let mut frames = Vec::with_capacity(total_frames);

        for frame_idx in 0..total_frames {
            // Update characters
            {
                let characters = terminal.get_characters_mut();
                for character in characters.iter_mut() {
                    // Each character's id directly indexes final_coords
                    let final_coord = final_coords[character.id as usize];
                    let start_frame = (final_coord.x as usize) * frames_per_column;
                    let raw_progress =
                        (frame_idx as f64 - start_frame as f64) / move_duration as f64;

                    if raw_progress <= 0.0 {
                        // Not yet started, keep hidden at off-screen position
                        character.position = Coord::new(-1, final_coord.y);
                        character.visible = false;
                    } else if raw_progress >= 1.0 {
                        // Finished: exactly at final position, final color
                        character.position = final_coord;
                        character.visible = true;
                        character.color_pair.fg = final_color;
                        character.color_pair.bg = bg_color;
                    } else {
                        // In motion
                        let eased_progress = cubic_out.ease(raw_progress.min(1.0).max(0.0));
                        let current_x = -1.0 + (final_coord.x as f64 + 1.0) * eased_progress;
                        let current_y = final_coord.y as f64; // y remains constant in sweep
                        character.position = Coord::new(
                            current_x.round() as i32,
                            current_y.round() as i32,
                        );
                        character.visible = true;
                        character.color_pair.fg = gradient.color_at(eased_progress);
                        character.color_pair.bg = bg_color;
                    }
                }
            }

            // Render current frame
            frames.push(terminal.render_frame());
        }

        frames
    }
}
