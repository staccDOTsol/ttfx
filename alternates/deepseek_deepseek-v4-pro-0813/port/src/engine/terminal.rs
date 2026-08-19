use crate::engine::canvas::Canvas;
use crate::engine::character::EffectCharacter;
use crate::utils::geometry::Coord;
use std::collections::HashMap;

/// Terminal configuration and state.
pub struct Terminal {
    pub canvas: Canvas,
    pub characters: Vec<EffectCharacter>,
    pub character_map: HashMap<u32, usize>, // character_id -> index in characters
}

impl Terminal {
    pub fn new(width: u16, height: u16) -> Self {
        Terminal {
            canvas: Canvas::new(width, height),
            characters: Vec::new(),
            character_map: HashMap::new(),
        }
    }

    pub fn add_character(&mut self, character: EffectCharacter) {
        let id = character.id;
        self.characters.push(character);
        self.character_map.insert(id, self.characters.len() - 1);
    }

    pub fn get_character(&self, id: u32) -> Option<&EffectCharacter> {
        self.character_map
            .get(&id)
            .and_then(|&idx| self.characters.get(idx))
    }

    pub fn get_character_mut(&mut self, id: u32) -> Option<&mut EffectCharacter> {
        if let Some(&idx) = self.character_map.get(&id) {
            self.characters.get_mut(idx)
        } else {
            None
        }
    }

    pub fn set_character_visibility(&mut self, id: u32, visible: bool) {
        if let Some(character) = self.get_character_mut(id) {
            character.visible = visible;
        }
    }

    pub fn get_characters(&self) -> &[EffectCharacter] {
        &self.characters
    }

    pub fn get_characters_mut(&mut self) -> &mut [EffectCharacter] {
        &mut self.characters
    }

    pub fn render_frame(&self) -> String {
        self.canvas.render(&self.characters)
    }
}
