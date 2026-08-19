use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::{Color, ColorPair, Coord, Gradient};

/// Rain effect: input characters fall from the top of the terminal as
/// blue/cyan raindrops to their input positions.
pub struct Rain;

impl Rain {
    pub fn new() -> Self {
        Rain
    }
}

impl Effect for Rain {
    fn name(&self) -> &str {
        "rain"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let text: Vec<char> = input.chars().filter(|c| !c.is_control()).collect();
        if text.is_empty() {
            return Vec::new();
        }

        let width = text.len() as u16;
        let height = 24u16;
        let target_row = (height / 2) as i32;
        let max_delay = 8i32;

        // Rain-colored vertical gradient: bright cyan at top, deep blue at bottom.
        let rain_gradient = Gradient::new(vec![
            (0.0, Color::new(0, 191, 255)),
            (0.5, Color::new(30, 144, 255)),
            (1.0, Color::new(0, 0, 139)),
        ]);

        let mut terminal = Terminal::new(width, height);

        for (i, &symbol) in text.iter().enumerate() {
            let mut character = EffectCharacter::new(i as u32, Coord::new(i as i32, 0), symbol);
            character.bold = true;
            terminal.add_character(character);
        }

        let mut frames = Vec::new();

        for step in 0..=(target_row + max_delay) {
            {
                let characters = terminal.get_characters_mut();
                for (i, character) in characters.iter_mut().enumerate() {
                    let delay = ((i as i32 * 7) % (max_delay + 1)) as i32;
                    let y = if step < delay {
                        -1
                    } else {
                        (step - delay).min(target_row)
                    };

                    character.position = Coord::new(i as i32, y);
                    character.visible = y >= 0;

                    let progress = if y <= 0 {
                        0.0
                    } else {
                        y as f64 / target_row as f64
                    };

                    let fg = rain_gradient.color_at(progress);
                    character.color_pair = ColorPair::new(fg, Color::BLACK);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
