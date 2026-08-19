//! Scenes, frames, and per-character animation state.

use std::collections::HashMap;

use crate::utils::graphics::ColorPair;

/// The styled appearance of a character for a single frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterVisual {
    pub symbol: char,
    pub colors: Option<ColorPair>,
    pub bold: bool,
}

impl CharacterVisual {
    pub fn new(symbol: char, colors: Option<ColorPair>) -> Self {
        Self { symbol, colors, bold: false }
    }

    pub fn plain(symbol: char) -> Self {
        Self::new(symbol, None)
    }

    /// The symbol with ANSI SGR sequences applied.
    pub fn formatted(&self) -> String {
        let mut needs_reset = false;
        let mut out = String::new();
        if self.bold {
            out.push_str("\x1b[1m");
            needs_reset = true;
        }
        if let Some(colors) = &self.colors {
            if let Some(fg) = colors.fg {
                out.push_str(&format!("\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b));
                needs_reset = true;
            }
            if let Some(bg) = colors.bg {
                out.push_str(&format!("\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b));
                needs_reset = true;
            }
        }
        out.push(self.symbol);
        if needs_reset {
            out.push_str("\x1b[0m");
        }
        out
    }
}

/// A single frame in a scene: a visual shown for `duration` ticks.
#[derive(Debug, Clone)]
pub struct Frame {
    pub visual: CharacterVisual,
    pub duration: u32,
    pub ticks_elapsed: u32,
}

/// An ordered sequence of frames, optionally looping.
#[derive(Debug, Clone)]
pub struct Scene {
    pub scene_id: String,
    pub frames: Vec<Frame>,
    pub current_frame_index: usize,
    pub is_looping: bool,
}

impl Scene {
    pub fn new(scene_id: &str, is_looping: bool) -> Self {
        Self {
            scene_id: scene_id.to_string(),
            frames: Vec::new(),
            current_frame_index: 0,
            is_looping,
        }
    }

    pub fn add_frame(&mut self, symbol: char, duration: u32, colors: Option<ColorPair>) {
        self.frames.push(Frame {
            visual: CharacterVisual::new(symbol, colors),
            duration: duration.max(1),
            ticks_elapsed: 0,
        });
    }

    pub fn reset(&mut self) {
        self.current_frame_index = 0;
        for frame in &mut self.frames {
            frame.ticks_elapsed = 0;
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.is_looping && self.current_frame_index >= self.frames.len()
    }

    /// Advance the scene one tick and return the visual for this tick.
    pub fn step(&mut self) -> CharacterVisual {
        if self.frames.is_empty() {
            return CharacterVisual::plain(' ');
        }
        if self.current_frame_index >= self.frames.len() {
            return self.frames.last().expect("non-empty frames").visual.clone();
        }
        let len = self.frames.len();
        let frame = &mut self.frames[self.current_frame_index];
        let visual = frame.visual.clone();
        frame.ticks_elapsed += 1;
        if frame.ticks_elapsed >= frame.duration {
            frame.ticks_elapsed = 0;
            self.current_frame_index += 1;
            if self.current_frame_index >= len && self.is_looping {
                self.current_frame_index = 0;
            }
        }
        visual
    }
}

/// Animation state for a single character: a set of scenes and the current visual.
#[derive(Debug, Clone)]
pub struct Animation {
    pub scenes: HashMap<String, Scene>,
    pub active_scene_id: Option<String>,
    pub current_visual: CharacterVisual,
}

impl Animation {
    pub fn new(input_symbol: char) -> Self {
        Self {
            scenes: HashMap::new(),
            active_scene_id: None,
            current_visual: CharacterVisual::plain(input_symbol),
        }
    }

    pub fn new_scene(&mut self, scene_id: &str, is_looping: bool) -> &mut Scene {
        self.scenes
            .entry(scene_id.to_string())
            .or_insert_with(|| Scene::new(scene_id, is_looping))
    }

    pub fn query_scene(&mut self, scene_id: &str) -> Option<&mut Scene> {
        self.scenes.get_mut(scene_id)
    }

    pub fn activate_scene(&mut self, scene_id: &str) {
        if let Some(scene) = self.scenes.get_mut(scene_id) {
            scene.reset();
            self.active_scene_id = Some(scene_id.to_string());
        }
    }

    pub fn deactivate_scene(&mut self) {
        self.active_scene_id = None;
    }

    pub fn active_scene_is_complete(&self) -> bool {
        match &self.active_scene_id {
            Some(id) => self.scenes.get(id).map(Scene::is_complete).unwrap_or(true),
            None => true,
        }
    }

    /// Directly set the character's appearance (bypassing scenes).
    pub fn set_appearance(&mut self, symbol: char, colors: Option<ColorPair>) {
        self.current_visual = CharacterVisual::new(symbol, colors);
    }

    /// Advance the active scene one tick, updating the current visual.
    pub fn step_animation(&mut self) {
        if let Some(id) = self.active_scene_id.clone() {
            let mut finished = false;
            if let Some(scene) = self.scenes.get_mut(&id) {
                self.current_visual = scene.step();
                finished = scene.is_complete();
            }
            if finished {
                self.active_scene_id = None;
            }
        }
    }
}
