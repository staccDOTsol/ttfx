use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Overflow {
    frame_count: usize,
}

impl Overflow {
    pub fn new() -> Self {
        Self { frame_count: 40 }
    }
}

impl Effect for Overflow {
    fn name(&self) -> &str {
        "overflow"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let line_count = lines.len().max(1);
        let max_line_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);

        let padding = 4usize;
        let width = (max_line_len + padding * 2).clamp(1, 120) as u16;
        let height = (line_count + padding * 2).clamp(1, 40) as u16;

        let mut terminal = Terminal::new(width, height);

        let start_x_offset = ((width as usize).saturating_sub(max_line_len) / 2) as i32;
        let start_y_offset = ((height as usize).saturating_sub(line_count) / 2) as i32;

        let mut starts = Vec::new();
        let mut targets = Vec::new();
        let mut id: u32 = 0;

        let center = Coord::new((width / 2) as i32, (height / 2) as i32);

        for (row_idx, line) in lines.iter().enumerate() {
            for (col_idx, ch) in line.chars().enumerate() {
                let start = Coord::new(
                    start_x_offset + col_idx as i32,
                    start_y_offset + row_idx as i32,
                );
                let mut character = EffectCharacter::new(id, start, ch);
                character.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
                terminal.add_character(character);
                starts.push(start);

                let dir_x = (start.x - center.x) as f64;
                let dir_y = (start.y - center.y) as f64;
                let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
                let (dx, dy) = if len < 0.001 {
                    (1.0_f64, 1.0_f64)
                } else {
                    (dir_x / len, dir_y / len)
                };
                let dist = (width.max(height)) as f64 * 1.5;
                let target = Coord::new(
                    (start.x as f64 + dx * dist).round() as i32,
                    (start.y as f64 + dy * dist).round() as i32,
                );
                targets.push(target);
                id += 1;
            }
        }

        if terminal.get_characters().is_empty() {
            let start = Coord::new((width / 2) as i32, (height / 2) as i32);
            let mut character = EffectCharacter::new(id, start, ' ');
            character.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
            terminal.add_character(character);
            starts.push(start);
            targets.push(start);
        }

        let mut frames = Vec::new();
        let gradient = Gradient::new(vec![
            (0.0, Color::new(255, 255, 255)),
            (0.25, Color::new(0, 255, 255)),
            (0.5, Color::new(255, 0, 255)),
            (0.75, Color::new(255, 165, 0)),
            (1.0, Color::new(255, 0, 0)),
        ]);

        for frame_idx in 0..self.frame_count {
            let p = frame_idx as f64 / (self.frame_count - 1).max(1) as f64;
            let eased = 1.0 - (1.0 - p).powi(3); // ease-out cubic
            let color = gradient.color_at(eased);

            for (i, character) in terminal.get_characters_mut().iter_mut().enumerate() {
                if let Some(&start) = starts.get(i) {
                    let target = targets[i];
                    let x = start.x as f64 + (target.x - start.x) as f64 * eased;
                    let y = start.y as f64 + (target.y - start.y) as f64 * eased;
                    character.position = Coord::new(x.round() as i32, y.round() as i32);
                    character.color_pair = ColorPair::new(color, Color::BLACK);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
