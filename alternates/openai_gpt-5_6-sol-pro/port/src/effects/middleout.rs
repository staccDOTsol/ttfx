use super::Effect;
use crate::engine::{
    CharacterId, CharacterVisual, EffectCharacter, Frame, Path, Scene,
    Terminal, Waypoint,
};
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

const CENTER_PATH: &str = "middleout_center";
const FULL_PATH: &str = "middleout_full";
const FULL_SCENE: &str = "middleout_gradient";

pub struct Middleout;

impl Middleout {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Middleout {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Middleout {
    fn name(&self) -> &str {
        "middleout"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        let text = normalized.trim_end_matches('\n');
        let lines: Vec<&str> = if text.is_empty() {
            vec![""]
        } else {
            text.split('\n').collect()
        };

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let center = Coord::new(
            (width.saturating_sub(1) / 2) as i32,
            (height.saturating_sub(1) / 2) as i32,
        );

        let starting_color = Color::new(255, 255, 255);
        let final_colors = Gradient::new(
            [
                Color::new(138, 0, 138),
                Color::new(0, 209, 255),
                Color::new(255, 255, 255),
            ],
            height,
        )
        .colors();

        let mut terminal = Terminal::new(width, height);
        let mut next_id = 0_u32;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let input_coord = Coord::new(column as i32, row as i32);
                let center_line_coord =
                    Coord::new(input_coord.column, center.row);
                let final_color = final_colors
                    .get(row)
                    .copied()
                    .unwrap_or(starting_color);

                let mut character = EffectCharacter::new(
                    CharacterId(next_id),
                    symbol.to_string(),
                    center,
                );
                next_id += 1;

                character.style = Style::with_colors(ColorPair::new(
                    Some(starting_color),
                    None,
                ));

                let mut center_path = Path::new(CENTER_PATH, 0.35);
                center_path.add_waypoint(Waypoint::new(
                    "center",
                    center_line_coord,
                ));
                character.motion.add_path(center_path);

                let mut full_path = Path::new(FULL_PATH, 0.35);
                full_path.add_waypoint(Waypoint::new("input", input_coord));
                character.motion.add_path(full_path);

                let mut gradient_scene = Scene::new(FULL_SCENE, false);
                for color in Gradient::new(
                    [starting_color, final_color],
                    12,
                )
                .colors()
                {
                    gradient_scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            symbol.to_string(),
                            Style::with_colors(ColorPair::new(
                                Some(color),
                                None,
                            )),
                        ),
                        1,
                    ));
                }
                character.animation.add_scene(gradient_scene);
                character.motion.activate(CENTER_PATH, center);

                terminal.add_character(character);
            }
        }

        if terminal.characters().is_empty() {
            let mut placeholder =
                EffectCharacter::new(CharacterId(0), " ", center);
            placeholder.style = Style::with_colors(ColorPair::new(
                Some(starting_color),
                None,
            ));
            terminal.add_character(placeholder);
        }

        // 0: moving from the central point to the center line
        // 1: expanding from the center line to the input coordinate
        // 2: complete
        let mut stages = vec![0_u8; terminal.characters().len()];
        let mut frames = Vec::new();
        let frame_limit = width
            .saturating_add(height)
            .saturating_mul(20)
            .saturating_add(64);

        for _ in 0..frame_limit {
            frames.push(terminal.step_frame());

            for (character, stage) in terminal
                .characters_mut()
                .iter_mut()
                .zip(stages.iter_mut())
            {
                match *stage {
                    0 if !character.motion.is_active() => {
                        character
                            .motion
                            .activate(FULL_PATH, character.position);
                        character.animation.activate(FULL_SCENE);
                        *stage = 1;
                    }
                    1 if !character.motion.is_active()
                        && !character.animation.is_active() =>
                    {
                        *stage = 2;
                    }
                    _ => {}
                }
            }

            if stages.iter().all(|stage| *stage == 2) {
                break;
            }
        }

        frames
    }
}
