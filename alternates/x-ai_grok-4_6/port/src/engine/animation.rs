use crate::utils::graphics::{Color, ColorPair};

#[derive(Clone, Debug, Default)]
pub struct CharacterVisual {
    pub symbol: char,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strike: bool,
    pub colors: Option<ColorPair>,
}

impl CharacterVisual {
    pub fn format_symbol(&self) -> String {
        self.symbol.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub visual: CharacterVisual,
    pub duration: u32,
    remaining: u32,
}

impl Frame {
    pub fn new(visual: CharacterVisual, duration: u32) -> Self {
        Self {
            visual,
            duration: duration.max(1),
            remaining: duration.max(1),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub id: String,
    pub frames: Vec<Frame>,
    pub is_looping: bool,
    index: usize,
}

impl Scene {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            frames: Vec::new(),
            is_looping: false,
            index: 0,
        }
    }

    pub fn add_frame(&mut self, symbol: char, duration: u32, colors: Option<ColorPair>) {
        let visual = CharacterVisual {
            symbol,
            colors,
            ..Default::default()
        };
        self.frames.push(Frame::new(visual, duration));
    }

    pub fn step(&mut self) -> Option<&CharacterVisual> {
        if self.frames.is_empty() {
            return None;
        }
        let frame = &mut self.frames[self.index];
        if frame.remaining > 0 {
            frame.remaining -= 1;
        }
        if frame.remaining == 0 {
            frame.remaining = frame.duration;
            if self.index + 1 < self.frames.len() {
                self.index += 1;
            } else if self.is_looping {
                self.index = 0;
            }
        }
        Some(&self.frames[self.index].visual)
    }
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub active_scene: Option<String>,
    pub current_character_visual: CharacterVisual,
    scenes: Vec<Scene>,
}

impl Animation {
    pub fn new(symbol: char) -> Self {
        Self {
            active_scene: None,
            current_character_visual: CharacterVisual {
                symbol,
                ..Default::default()
            },
            scenes: Vec::new(),
        }
    }

    pub fn new_scene(&mut self, id: impl Into<String>) -> &mut Scene {
        self.scenes.push(Scene::new(id));
        self.scenes.last_mut().unwrap()
    }

    pub fn query_scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.iter().find(|s| s.id == id)
    }

    pub fn activate_scene(&mut self, id: &str) {
        if self.scenes.iter().any(|s| s.id == id) {
            self.active_scene = Some(id.to_string());
        }
    }

    pub fn set_appearance(&mut self, symbol: char, color: Option<Color>) {
        self.current_character_visual.symbol = symbol;
        if let Some(c) = color {
            self.current_character_visual.colors = Some(ColorPair {
                fg: Some(c),
                bg: None,
            });
        }
    }

    pub fn step_animation(&mut self) {
        if let Some(id) = self.active_scene.clone() {
            if let Some(scene) = self.scenes.iter_mut().find(|s| s.id == id) {
                if let Some(vis) = scene.step() {
                    self.current_character_visual = vis.clone();
                }
            }
        }
    }
}
