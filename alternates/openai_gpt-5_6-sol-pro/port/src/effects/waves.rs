use super::Effect;
use crate::engine::{
    CharacterId, CharacterVisual, EffectCharacter, Frame, Scene, Terminal,
};
use crate::utils::graphics::{Color, ColorPair, Gradient, Style};
use crate::utils::Coord;

pub struct Waves;

impl Waves {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Waves {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Waves {
    fn name(&self) -> &str {
        "waves"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        const WAVE_SYMBOLS: [&str; 8] = [
            "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█",
        ];
        const WAVE_STOPS: [Color; 5] = [
            Color::new(0xf0, 0xff, 0x65),
            Color::new(0x65, 0xff, 0xb4),
            Color::new(0x65, 0xc7, 0xff),
            Color::new(0x65, 0x65, 0xff),
            Color::new(0xff, 0x65, 0xf4),
        ];
        const WAVE_COUNT: usize = 2;
        const WAVE_LENGTH: u32 = 2;

        let lines: Vec<Vec<char>> = input
            .lines()
            .map(|line| {
                line.strip_suffix('\r')
                    .unwrap_or(line)
                    .chars()
                    .collect()
            })
            .collect();

        let width = lines.iter().map(Vec::len).max().unwrap_or(0);
        let height = lines.len();

        if width == 0 || height == 0 {
            return Vec::new();
        }

        let wave_colors =
            Gradient::new(WAVE_STOPS, WAVE_SYMBOLS.len()).colors();
        let final_colors = Gradient::new(WAVE_STOPS, height.max(2)).colors();

        let mut terminal = Terminal::new(width, height);
        let mut activation_frames = Vec::new();
        let mut next_id = 0_u32;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.iter().enumerate() {
                let position = Coord::new(column as i32, row as i32);
                let mut character = EffectCharacter::new(
                    CharacterId(next_id),
                    symbol.to_string(),
                    position,
                );
                next_id += 1;
                character.visible = false;

                let final_color = final_colors
                    .get(row)
                    .copied()
                    .unwrap_or(WAVE_STOPS[WAVE_STOPS.len() - 1]);
                let mut scene = Scene::new("wave", false);

                for _ in 0..WAVE_COUNT {
                    for (index, wave_symbol) in
                        WAVE_SYMBOLS.iter().enumerate()
                    {
                        let color = wave_colors
                            .get(index)
                            .copied()
                            .unwrap_or(WAVE_STOPS[0]);
                        let style = Style::with_colors(ColorPair::new(
                            Some(color),
                            None,
                        ));

                        scene.add_frame(Frame::new(
                            CharacterVisual::new(*wave_symbol, style),
                            WAVE_LENGTH,
                        ));
                    }
                }

                let final_style = Style::with_colors(ColorPair::new(
                    Some(final_color),
                    None,
                ));
                scene.add_frame(Frame::new(
                    CharacterVisual::new(symbol.to_string(), final_style),
                    1,
                ));

                character.animation.add_scene(scene);
                terminal.add_character(character);

                // Activate one column per frame, producing a wave that travels
                // from the left side of the text to the right.
                activation_frames.push(column);
            }
        }

        let last_activation = activation_frames
            .iter()
            .copied()
            .max()
            .unwrap_or(0);
        let mut frames = Vec::new();
        let mut tick = 0_usize;

        loop {
            for (character, activation_frame) in terminal
                .characters_mut()
                .iter_mut()
                .zip(activation_frames.iter().copied())
            {
                if activation_frame == tick {
                    character.visible = true;
                    character.animation.activate("wave");
                }
            }

            frames.push(terminal.step_frame());

            if tick >= last_activation && !terminal.has_active_characters() {
                break;
            }

            tick += 1;
        }

        frames
    }
}
