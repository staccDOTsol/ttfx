//! Terminal: owns the character arena and the canvas, produces frames.

use crate::engine::canvas::Canvas;
use crate::engine::character::EffectCharacter;
use crate::utils::geometry::Coord;

/// Terminal configuration.
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    pub width: i32,
    pub height: i32,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { width: 80, height: 24 }
    }
}

/// The simulation terminal: character arena plus render canvas.
#[derive(Debug, Clone)]
pub struct Terminal {
    pub config: TerminalConfig,
    pub canvas: Canvas,
    characters: Vec<EffectCharacter>,
    next_character_id: u32,
}

impl Terminal {
    /// Build a terminal from input text. The top line of the input is placed
    /// on the top row of the canvas; row 1 is the bottom row.
    pub fn new(input: &str, config: TerminalConfig) -> Self {
        let lines: Vec<&str> = input.lines().collect();
        let width = config
            .width
            .max(lines.iter().map(|l| l.chars().count() as i32).max().unwrap_or(1))
            .max(1);
        let height = config.height.max(lines.len() as i32).max(1);
        let mut terminal = Self {
            config: TerminalConfig { width, height },
            canvas: Canvas::new(width, height),
            characters: Vec::new(),
            next_character_id: 0,
        };
        for (line_index, line) in lines.iter().enumerate() {
            let row = height - line_index as i32;
            if row < 1 {
                break;
            }
            for (col_index, symbol) in line.chars().enumerate() {
                let column = col_index as i32 + 1;
                if column > width {
                    break;
                }
                if symbol != ' ' {
                    terminal.add_character(symbol, Coord::new(column, row));
                }
            }
        }
        terminal
    }

    pub fn add_character(&mut self, symbol: char, coord: Coord) -> u32 {
        let id = self.next_character_id;
        self.next_character_id += 1;
        self.characters.push(EffectCharacter::new(id, symbol, coord));
        id
    }

    pub fn get_characters(&self) -> &[EffectCharacter] {
        &self.characters
    }

    pub fn get_characters_mut(&mut self) -> &mut [EffectCharacter] {
        &mut self.characters
    }

    pub fn set_character_visibility(&mut self, character_id: u32, is_visible: bool) {
        if let Some(character) = self
            .characters
            .iter_mut()
            .find(|c| c.character_id == character_id)
        {
            character.is_visible = is_visible;
        }
    }

    /// True while any character still has work remaining.
    pub fn is_active(&self) -> bool {
        self.characters.iter().any(EffectCharacter::is_active)
    }

    /// Advance every character by one tick.
    pub fn tick(&mut self) {
        for character in &mut self.characters {
            character.tick();
        }
    }

    /// Composite visible characters onto the canvas and render a frame string.
    pub fn render_frame(&mut self) -> String {
        self.canvas.clear();
        for character in &self.characters {
            if character.is_visible {
                self.canvas
                    .set_cell(character.motion.current_coord, character.animation.current_visual.clone());
            }
        }
        self.canvas.to_frame_string()
    }

    /// Run the frame-stepping loop to completion (bounded by `max_frames`),
    /// collecting one rendered frame per tick.
    pub fn run(&mut self, max_frames: usize) -> Vec<String> {
        let mut frames = Vec::new();
        frames.push(self.render_frame());
        let mut count = 0;
        while self.is_active() && count < max_frames {
            self.tick();
            frames.push(self.render_frame());
            count += 1;
        }
        frames
    }
}
