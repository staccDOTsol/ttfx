use crate::engine::animation::{Animation, CharacterVisual};
use crate::engine::motion::Motion;
use crate::utils::geometry::Coord;
use crate::utils::graphics::Style;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CharacterId(pub u32);

#[derive(Clone, Debug)]
pub struct EffectCharacter {
    pub id: CharacterId,
    pub input_symbol: String,
    pub symbol: String,
    pub position: Coord,
    pub visible: bool,
    pub style: Style,
    pub animation: Animation,
    pub motion: Motion,
}

impl EffectCharacter {
    pub fn new(
        id: CharacterId,
        symbol: impl Into<String>,
        position: Coord,
    ) -> Self {
        let symbol = symbol.into();

        Self {
            id,
            input_symbol: symbol.clone(),
            symbol,
            position,
            visible: true,
            style: Style::default(),
            animation: Animation::default(),
            motion: Motion::default(),
        }
    }

    pub fn set_appearance(&mut self, visual: CharacterVisual) {
        self.symbol = visual.symbol;
        self.style = visual.style;
    }

    pub fn reset_appearance(&mut self) {
        self.symbol.clone_from(&self.input_symbol);
        self.style = Style::default();
    }

    pub fn step(&mut self) {
        if let Some(position) = self.motion.step(self.position) {
            self.position = position;
        }

        if let Some(visual) = self.animation.step() {
            self.set_appearance(visual);
        }
    }

    pub fn is_active(&self) -> bool {
        self.motion.is_active() || self.animation.is_active()
    }
}
