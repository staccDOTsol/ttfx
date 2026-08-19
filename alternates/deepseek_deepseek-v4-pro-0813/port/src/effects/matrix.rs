
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use crate::utils::easing::{self, Easing};
use super::Effect;

/// The Matrix digital rain effect.
pub struct Matrix;

impl Matrix {
    pub fn new() -> Self {
        Matrix
    }
}

impl Effect for Matrix {
    fn name(&self) -> &str {
        "matrix"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Parse input into grid
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);
        
        let mut terminal = Terminal::new(width as u16, height as u16);
        
        // Collect characters in row-major order
        let mut chars: Vec<EffectCharacter> = Vec::new();
        let mut id_counter: u32 = 0;
        
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let character = EffectCharacter::new(
                        id_counter,
                        Coord::new(col as i32, row as i32),
                        ch,
                    );
                    chars.push(character);
                    id_counter += 1;
                }
            }
        }
        
        // Add characters to terminal
        for character in chars {
            terminal.add_character(character);
        }
        
        // Get total steps for animation
        let total_steps = 60;
        let mut frames = Vec::new();
        
        // Green shades for matrix effect
        let green_shades = [
            Color::new(0, 40, 0),
            Color::new(0, 80, 0),
            Color::new(0, 120, 0),
            Color::new(0, 160, 0),
            Color::new(0, 200, 0),
            Color::new(0, 255, 0),
            Color::new(100, 255, 100),
            Color::new(180, 255, 180),
        ];
        
        let gradient = Gradient::new(vec![
            (0.0, Color::new(0, 255, 0)),
            (0.5, Color::new(0, 180, 0)),
            (1.0, Color::new(0, 60, 0)),
        ]);
        
        // Animation loop
        for frame_idx in 0..total_steps {
            // Update character positions and colors
            for char_idx in 0..terminal.get_characters().len() {
                if let Some(character) = terminal.get_character_mut(char_idx as u32) {
                    // Move characters downward with a wave effect
                    let speed = 0.5 + ((character.id as f64 * 0.7) % 1.0);
                    let new_y = (character.position.y as f64 + speed) % height as f64;
                    character.position.y = new_y as i32;
                    
                    // Calculate wave offset based on x position
                    let wave = ((character.position.x as f64 * 0.3) 
                        + (frame_idx as f64 * 0.2)).sin() * 2.0;
                    character.position.y = ((character.position.y as f64 + wave) % height as f64) as i32;
                    
                    // Color based on position and time
                    let color_idx = ((character.position.y as f64 / height as f64) 
                        * (green_shades.len() - 1) as f64) as usize;
                    let color = green_shades[color_idx.min(green_shades.len() - 1)];
                    
                    // Apply gradient fading
                    let t = (character.position.y as f64 / height as f64 + frame_idx as f64 * 0.02) % 1.0;
                    let gradient_color = gradient.color_at(t);
                    
                    // Brighten characters near the top
                    if character.position.y < height as i32 / 3 {
                        character.color_pair = ColorPair::new(
                            Color::new(180, 255, 180),
                            Color::BLACK,
                        );
                    } else {
                        character.color_pair = ColorPair::new(
                            gradient_color,
                            Color::BLACK,
                        );
                    }
                    
                    // Randomly brighten some characters to simulate the "leading" character
                    if (character.id.wrapping_mul(7).wrapping_add(frame_idx as u32 * 3)) % 11 == 0 {
                        character.color_pair = ColorPair::new(
                            Color::new(255, 255, 255),
                            Color::BLACK,
                        );
                    }
                    
                    // Set bold for some characters
                    character.bold = (character.id.wrapping_mul(13).wrapping_add(frame_idx as u32)) % 5 == 0;
                }
            }
            
            // Render the frame
            frames.push(terminal.render_frame());
        }
        
        frames
    }
}
