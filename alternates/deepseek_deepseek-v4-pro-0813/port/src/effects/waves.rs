use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use std::f64::consts::PI;

pub struct Waves {
    amplitude: i32,
    wavelength: f64,
    speed: f64,
    frame_count: i32,
}

impl Waves {
    pub fn new() -> Self {
        Self {
            amplitude: 2,
            wavelength: 8.0,
            speed: 0.4,
            frame_count: 120,
        }
    }

    fn wave_color(gradient: &Gradient, t: f64) -> Color {
        gradient.color_at(t.abs().sin())
    }
}

impl Effect for Waves {
    fn name(&self) -> &str {
        "waves"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let input = input.trim_matches(|c| c == '\n' || c == '\r');
        let input = if input.is_empty() {
            "TerminalTextEffects"
        } else {
            input
        };

        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(20)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut base_coords: Vec<Coord> = Vec::new();
        let mut id = 0u32;

        let wave_gradient = Gradient::new(vec![
            (0.0, Color::new(0, 255, 255)),
            (0.25, Color::new(0, 180, 255)),
            (0.5, Color::new(128, 0, 255)),
            (0.75, Color::new(255, 0, 200)),
            (1.0, Color::new(255, 255, 255)),
        ]);

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let coord = Coord::new(col as i32, row as i32);
                let mut character = EffectCharacter::new(id, coord, ch);

                let t = (col as f64 / width.max(1) as f64) * 0.7
                    + (row as f64 / height.max(1) as f64) * 0.3;
                character.color_pair =
                    ColorPair::new(Self::wave_color(&wave_gradient, t), Color::BLACK);

                terminal.add_character(character);
                base_coords.push(coord);
                id += 1;
            }
        }

        if terminal.get_characters().is_empty() {
            for (col, ch) in "TerminalTextEffects".chars().enumerate() {
                let coord = Coord::new(col as i32, 0);
                let mut character = EffectCharacter::new(id, coord, ch);
                let t = col as f64 / 20.0;
                character.color_pair =
                    ColorPair::new(Self::wave_color(&wave_gradient, t), Color::BLACK);
                terminal.add_character(character);
                base_coords.push(coord);
                id += 1;
            }
        }

        let mut frames = Vec::with_capacity(self.frame_count as usize);

        for frame_idx in 0..self.frame_count {
            let time = frame_idx as f64 * self.speed;

            {
                let characters = terminal.get_characters_mut();
                for (i, base) in base_coords.iter().enumerate() {
                    let x = base.x;
                    let wave_y = (x as f64 * 2.0 * PI / self.wavelength + time).sin()
                        * self.amplitude as f64;
                    let new_y = base.y + wave_y.round() as i32;

                    let in_bounds =
                        x >= 0 && x < width as i32 && new_y >= 0 && new_y < height as i32;

                    let t = ((x as f64 / width.max(1) as f64) * 0.6
                        + (new_y as f64 / height.max(1) as f64) * 0.4
                        + frame_idx as f64 * 0.03)
                        .sin()
                        .abs();

                    characters[i].position = Coord::new(x, new_y);
                    characters[i].visible = in_bounds;
                    characters[i].color_pair =
                        ColorPair::new(Self::wave_color(&wave_gradient, t), Color::BLACK);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
