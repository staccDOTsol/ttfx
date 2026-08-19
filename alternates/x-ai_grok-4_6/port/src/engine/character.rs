use crate::engine::animation::Animation;
use crate::engine::motion::Motion;
use crate::utils::geometry::Coord;
use crate::utils::graphics::ColorPair;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CharacterId(pub u32);

#[derive(Clone, Debug)]
pub struct EffectCharacter {
    pub id: CharacterId,
    pub input_symbol: char,
    pub input_coord: Coord,
    pub current_coord: Coord,
    pub colors: Option<ColorPair>,
    pub is_visible: bool,
    pub animation: Animation,
    pub motion: Motion,
}

impl EffectCharacter {
    pub fn new(id: CharacterId, symbol: char, coord: Coord) -> Self {
        Self {
            id,
            input_symbol: symbol,
            input_coord: coord,
            current_coord: coord,
            colors: None,
            is_visible: false,
            animation: Animation::new(symbol),
            motion: Motion::new(coord),
        }
    }
}
