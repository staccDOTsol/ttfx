use crate::engine::canvas::Canvas;
use crate::engine::character::{CharacterId, EffectCharacter};
use crate::utils::geometry::Coord;

#[derive(Clone, Debug)]
pub struct TerminalConfig {
    pub frame_rate: u32,
    pub existing_color_handling: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            frame_rate: 60,
            existing_color_handling: "always".into(),
        }
    }
}

#[derive(Debug)]
pub struct Terminal {
    pub canvas: Canvas,
    pub config: TerminalConfig,
    characters: Vec<EffectCharacter>,
    next_id: u32,
}

impl Terminal {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            canvas: Canvas::new(width, height),
            config: TerminalConfig::default(),
            characters: Vec::new(),
            next_id: 0,
        }
    }

    pub fn from_input(input: &str, width: usize, height: usize) -> Self {
        let mut term = Self::new(width, height);
        for (row, line) in input.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' && ch != '\t' {
                    let coord = Coord {
                        column: col as i32,
                        row: row as i32,
                    };
                    term.add_character(ch, coord);
                }
            }
        }
        term
    }

    pub fn add_character(&mut self, symbol: char, coord: Coord) -> CharacterId {
        let id = CharacterId(self.next_id);
        self.next_id += 1;
        self.characters
            .push(EffectCharacter::new(id, symbol, coord));
        id
    }

    pub fn get_characters(&self) -> &[EffectCharacter] {
        &self.characters
    }

    pub fn get_characters_mut(&mut self) -> &mut [EffectCharacter] {
        &mut self.characters
    }

    pub fn set_character_visibility(&mut self, id: CharacterId, is_visible: bool) {
        if let Some(ch) = self.characters.iter_mut().find(|c| c.id == id) {
            ch.is_visible = is_visible;
        }
    }

    pub fn get_formatted_output_string(&mut self) -> String {
        self.canvas.fill(' ');
        for ch in &self.characters {
            if ch.is_visible {
                self.canvas
                    .set_symbol(ch.current_coord, ch.animation.current_character_visual.symbol);
            }
        }
        self.canvas.render()
    }

    pub fn step_all(&mut self) {
        for ch in &mut self.characters {
            ch.animation.step_animation();
            ch.motion.move_character();
            ch.current_coord = ch.motion.current_coord;
        }
    }
}
