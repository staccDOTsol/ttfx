//! EffectCharacter: one input character with animation and motion state.

use crate::engine::animation::Animation;
use crate::engine::motion::Motion;
use crate::utils::geometry::Coord;

/// Stable identifier for a character; allocation order matches input order.
pub type CharacterId = u32;

/// A single character from the input, with its home coordinate,
/// visibility, animation, and motion state.
#[derive(Debug, Clone)]
pub struct EffectCharacter {
    pub character_id: CharacterId,
    pub input_symbol: char,
    pub input_coord: Coord,
    pub is_visible: bool,
    pub animation: Animation,
    pub motion: Motion,
}

impl EffectCharacter {
    pub fn new(character_id: CharacterId, input_symbol: char, input_coord: Coord) -> Self {
        Self {
            character_id,
            input_symbol,
            input_coord,
            is_visible: false,
            animation: Animation::new(input_symbol),
            motion: Motion::new(input_coord),
        }
    }

    /// True while the character has motion or animation work remaining.
    pub fn is_active(&self) -> bool {
        !self.motion.movement_is_complete() || !self.animation.active_scene_is_complete()
    }

    /// Advance motion and animation by one tick.
    pub fn tick(&mut self) {
        self.motion.move_();
        self.animation.step_animation();
    }
}
