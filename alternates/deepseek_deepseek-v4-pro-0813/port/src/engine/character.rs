use crate::utils::geometry::Coord;
use crate::utils::graphics::ColorPair;

/// A single character in the effect, with position, symbol, and style.
pub struct EffectCharacter {
    pub id: u32,
    pub position: Coord,
    pub symbol: char,
    pub input_symbol: char,
    pub color_pair: ColorPair,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strike: bool,
    pub visible: bool,
}

impl EffectCharacter {
    pub fn new(id: u32, position: Coord, symbol: char) -> Self {
        EffectCharacter {
            id,
            position,
            symbol,
            input_symbol: symbol,
            color_pair: ColorPair::default(),
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strike: false,
            visible: true,
        }
    }
}
