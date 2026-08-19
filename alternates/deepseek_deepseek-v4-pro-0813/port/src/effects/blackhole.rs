use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use super::Effect;

pub struct Blackhole;

impl Blackhole {
    pub fn new() -> Self {
        Blackhole
    }
}

impl Effect for Blackhole {
    fn name(&self) -> &str {
        "blackhole"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let chars: Vec<char> = input
            .chars()
            .filter(|c| !c.is_control() || *c == ' ')
            .collect();
        let fallback: Vec<char> = "TerminalTextEffects".chars().collect();
        let chars = if chars.is_empty() { &fallback } else { &chars };
        let len = chars.len();

        let width = {
            let w = (len as f64).sqrt().ceil() as u16 + 2;
            if w < 10 { 10 } else { w }
        };
        let columns = (width - 2) as usize;
        let rows = (len + columns - 1) / columns;
        let height = (rows as u16 + 2).max(6);

        let mut terminal = Terminal::new(width, height);

        let center = Coord::new((width / 2) as i32, (height / 2) as i32);
        let max_x = (width - 1) as i32;
        let max_y = (height - 1) as i32;

        let mut starts = Vec::with_capacity(len);
        let mut dists = Vec::with_capacity(len);
        let mut angles = Vec::with_capacity(len);
        let mut color_offsets = Vec::with_capacity(len);

        for (i, ch) in chars.iter().copied().enumerate() {
            let col = 1 + (i % columns) as i32;
            let row = 1 + (i / columns) as i32;
            let coord = Coord::new(col.min(max_x), row.min(max_y));
            let dist = coord.distance(&center).max(1.0);
            let angle = i as f64 * 0.35 + dist * 0.15;
            let color_offset = (i % 5) as f64 * 0.04;

            starts.push(coord);
            dists.push(dist);
            angles.push(angle);
            color_offsets.push(color_offset);

            let character = EffectCharacter::new(i as u32, coord, ch);
            terminal.add_character(character);
        }

        let base_steps = 60 + ((width.max(height) / 2) as usize);
        let total_steps = base_steps.max(40);

        let fg_gradient = Gradient::new(vec![
            (0.0, Color::new(0, 230, 255)),
            (0.5, Color::new(255, 140, 0)),
            (0.85, Color::new(150, 0, 200)),
            (1.0, Color::BLACK),
        ]);

        let mut frames = Vec::with_capacity(total_steps);

        for step in 0..total_steps {
            let raw_p = step as f64 / (total_steps - 1) as f64;
            // smoothstep ease-in-out
            let p = raw_p * raw_p * (3.0 - 2.0 * raw_p);

            for (idx, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                let dist = dists[idx];
                let start_angle = angles[idx];
                let radius = dist * (1.0 - p);

                let angle = start_angle + p * 2.0 * std::f64::consts::PI * 2.0;
                let x = center.x + (radius * angle.cos()).round() as i32;
                let y = center.y + (radius * angle.sin()).round() as i32;

                character.position = Coord::new(
                    x.clamp(0, max_x),
                    y.clamp(0, max_y),
                );

                let color_p = (p + color_offsets[idx]).min(1.0);
                character.color_pair = ColorPair::new(fg_gradient.color_at(color_p), Color::BLACK);
                character.visible = p < 1.0;

                if p < 0.15 {
                    character.symbol = character.input_symbol;
                } else if p < 0.85 {
                    // Keep the original symbol for readability; more dramatic effects can
                    // swap symbols, but this keeps the text visible while moving.
                    character.symbol = character.input_symbol;
                } else {
                    character.symbol = '*';
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
