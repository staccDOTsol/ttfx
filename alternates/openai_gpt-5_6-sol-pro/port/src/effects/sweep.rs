
use super::Effect;
use crate::engine::{
    CharacterId, CharacterVisual, EffectCharacter, Frame, Scene, Terminal,
};
use crate::utils::{Color, Gradient, Style};

#[derive(Clone, Copy, Debug, Default)]
pub struct Sweep;

impl Sweep {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Sweep {
    fn name(&self) -> &str {
        "sweep"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<Vec<char>> = input
            .lines()
            .map(|line| line.trim_end_matches('\r').chars().collect())
            .collect();

        let width = lines.iter().map(Vec::len).max().unwrap_or(0);
        let height = lines.len();

        if width == 0 || height == 0 {
            return Vec::new();
        }

        let final_colors = Gradient::new(
            [
                Color::new(0x8a, 0x00, 0x8a),
                Color::new(0x00, 0xd1, 0xff),
                Color::new(0xff, 0xff, 0xff),
            ],
            8,
        )
        .colors();

        let sweep_color = Color::new(0xff, 0xff, 0xff);
        let sweep_symbols = ["█", "▓", "▒", "░"];
        let mut terminal = Terminal::new(width, height);
        let mut column_groups = vec![Vec::new(); width];
        let mut character_index = 0usize;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.iter().enumerate() {
                let final_color_index = if height <= 1 || final_colors.len() <= 1 {
                    0
                } else {
                    let inverted_row = height - 1 - row;
                    inverted_row * (final_colors.len() - 1) / (height - 1)
                };
                let final_color = final_colors[final_color_index];

                let sweep_colors =
                    Gradient::new([sweep_color, final_color], sweep_symbols.len())
                        .colors();

                let mut scene = Scene::new("sweep", false);

                for (sweep_symbol, color) in
                    sweep_symbols.iter().zip(sweep_colors.iter())
                {
                    scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            *sweep_symbol,
                            Style {
                                foreground: Some(*color),
                                ..Style::default()
                            },
                        ),
                        5,
                    ));
                }

                scene.add_frame(Frame::new(
                    CharacterVisual::new(
                        symbol.to_string(),
                        Style {
                            foreground: Some(final_color),
                            ..Style::default()
                        },
                    ),
                    1,
                ));

                let mut character = EffectCharacter::new(
                    CharacterId(character_index as u32),
                    symbol.to_string(),
                    crate::utils::Coord::new(column as i32, row as i32),
                );
                character.visible = false;
                character.animation.add_scene(scene);

                terminal.add_character(character);
                column_groups[column].push(character_index);
                character_index += 1;
            }
        }

        let mut frames = Vec::new();
        let mut next_group = 0usize;

        while next_group < column_groups.len()
            || terminal.has_active_characters()
        {
            if let Some(group) = column_groups.get(next_group) {
                let characters = terminal.characters_mut();

                for &index in group {
                    let character = &mut characters[index];
                    character.visible = true;
                    character.animation.activate("sweep");
                }

                next_group += 1;
            }

            frames.push(terminal.step_frame());
        }

        frames
    }
}
