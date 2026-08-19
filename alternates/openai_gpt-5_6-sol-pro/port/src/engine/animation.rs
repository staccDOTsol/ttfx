use std::collections::BTreeMap;

use crate::utils::graphics::Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterVisual {
    pub symbol: String,
    pub style: Style,
}

impl CharacterVisual {
    pub fn new(symbol: impl Into<String>, style: Style) -> Self {
        Self {
            symbol: symbol.into(),
            style,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub visual: CharacterVisual,
    pub duration: u32,
}

impl Frame {
    pub fn new(visual: CharacterVisual, duration: u32) -> Self {
        Self {
            visual,
            duration: duration.max(1),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub id: String,
    frames: Vec<Frame>,
    looping: bool,
    frame_index: usize,
    elapsed: u32,
    active: bool,
    complete: bool,
}

impl Scene {
    pub fn new(id: impl Into<String>, looping: bool) -> Self {
        Self {
            id: id.into(),
            frames: Vec::new(),
            looping,
            frame_index: 0,
            elapsed: 0,
            active: false,
            complete: false,
        }
    }

    pub fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn frames(&self) -> &[Frame] {
        &self.frames
    }

    pub fn activate(&mut self) -> bool {
        if self.frames.is_empty() {
            return false;
        }

        self.frame_index = 0;
        self.elapsed = 0;
        self.active = true;
        self.complete = false;
        true
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn current_visual(&self) -> Option<&CharacterVisual> {
        self.frames
            .get(self.frame_index)
            .map(|frame| &frame.visual)
    }

    pub fn step(&mut self) -> Option<CharacterVisual> {
        if !self.active || self.frames.is_empty() {
            return None;
        }

        let visual = self.frames[self.frame_index].visual.clone();
        self.elapsed += 1;

        if self.elapsed >= self.frames[self.frame_index].duration {
            self.elapsed = 0;

            if self.frame_index + 1 < self.frames.len() {
                self.frame_index += 1;
            } else if self.looping {
                self.frame_index = 0;
            } else {
                self.active = false;
                self.complete = true;
            }
        }

        Some(visual)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Animation {
    scenes: BTreeMap<String, Scene>,
    active_scene: Option<String>,
    current_visual: Option<CharacterVisual>,
}

impl Animation {
    pub fn add_scene(&mut self, scene: Scene) -> Option<Scene> {
        self.scenes.insert(scene.id.clone(), scene)
    }

    pub fn create_scene(
        &mut self,
        id: impl Into<String>,
        looping: bool,
    ) -> &mut Scene {
        let id = id.into();
        self.scenes
            .entry(id.clone())
            .or_insert_with(|| Scene::new(id, looping))
    }

    pub fn scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.get(id)
    }

    pub fn scene_mut(&mut self, id: &str) -> Option<&mut Scene> {
        self.scenes.get_mut(id)
    }

    pub fn activate(&mut self, id: &str) -> bool {
        if let Some(previous_id) = self.active_scene.take() {
            if let Some(previous) = self.scenes.get_mut(&previous_id) {
                previous.deactivate();
            }
        }

        let Some(scene) = self.scenes.get_mut(id) else {
            return false;
        };

        if !scene.activate() {
            return false;
        }

        self.current_visual = scene.current_visual().cloned();
        self.active_scene = Some(id.to_owned());
        true
    }

    pub fn deactivate(&mut self) {
        if let Some(id) = self.active_scene.take() {
            if let Some(scene) = self.scenes.get_mut(&id) {
                scene.deactivate();
            }
        }
    }

    pub fn current_visual(&self) -> Option<&CharacterVisual> {
        self.current_visual.as_ref()
    }

    pub fn is_active(&self) -> bool {
        self.active_scene
            .as_ref()
            .and_then(|id| self.scenes.get(id))
            .is_some_and(Scene::is_active)
    }

    pub fn step(&mut self) -> Option<CharacterVisual> {
        let id = self.active_scene.clone()?;
        let scene = self.scenes.get_mut(&id)?;
        let visual = scene.step();

        if let Some(visual) = &visual {
            self.current_visual = Some(visual.clone());
        }

        if !scene.is_active() {
            self.active_scene = None;
        }

        visual
    }
}
