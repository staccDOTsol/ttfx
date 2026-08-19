use crate::utils::graphics::ColorPair;

/// Visual representation of a character in a frame.
#[derive(Clone, Debug)]
pub struct CharacterVisual {
    pub symbol: char,
    pub color_pair: ColorPair,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strike: bool,
}

impl Default for CharacterVisual {
    fn default() -> Self {
        CharacterVisual {
            symbol: ' ',
            color_pair: ColorPair::default(),
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strike: false,
        }
    }
}

/// A single frame of animation: a grid of character visuals.
pub struct Frame {
    pub width: u16,
    pub height: u16,
    pub visuals: Vec<CharacterVisual>, // row-major
}

impl Frame {
    pub fn new(width: u16, height: u16) -> Self {
        Frame {
            width,
            height,
            visuals: vec![CharacterVisual::default(); (width as usize) * (height as usize)],
        }
    }

    pub fn set_visual(&mut self, x: u16, y: u16, visual: CharacterVisual) {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.visuals[idx] = visual;
        }
    }

    pub fn get_visual(&self, x: u16, y: u16) -> Option<&CharacterVisual> {
        if x < self.width && y < self.height {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            Some(&self.visuals[idx])
        } else {
            None
        }
    }
}

/// A sequence of frames.
pub struct Scene {
    pub frames: Vec<Frame>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { frames: Vec::new() }
    }

    pub fn add_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }
}

/// Animation state for a character, holding scenes and current frame index.
pub struct Animation {
    pub scenes: Vec<Scene>,
    pub current_scene: Option<usize>,
    pub current_frame: usize,
}

impl Animation {
    pub fn new() -> Self {
        Animation {
            scenes: Vec::new(),
            current_scene: None,
            current_frame: 0,
        }
    }

    pub fn add_scene(&mut self, scene: Scene) {
        self.scenes.push(scene);
        if self.current_scene.is_none() {
            self.current_scene = Some(self.scenes.len() - 1);
        }
    }

    pub fn step(&mut self) -> Option<&Frame> {
        if let Some(scene_idx) = self.current_scene {
            let scene = &self.scenes[scene_idx];
            if self.current_frame < scene.frames.len() {
                let frame = &scene.frames[self.current_frame];
                self.current_frame += 1;
                Some(frame)
            } else {
                None
            }
        } else {
            None
        }
    }
}
