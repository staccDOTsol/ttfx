use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::easing::{get_easing, Easing, Linear};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

#[derive(Clone, Copy)]
enum SliceDirection {
    Horizontal,
    Vertical,
    Diagonal,
}

pub struct Slice {
    direction: SliceDirection,
    speed: f64,
    easing_name: String,
}

impl Slice {
    pub fn new() -> Self {
        Self {
            direction: SliceDirection::Horizontal,
            speed: 2.0,
            easing_name: "cubic_out".to_string(),
        }
    }

    fn start_for(end: Coord, width: u16, height: u16, direction: SliceDirection) -> Coord {
        match direction {
            SliceDirection::Horizontal => {
                if end.x < width as i32 / 2 {
                    Coord::new(-1, end.y)
                } else {
                    Coord::new(width as i32, end.y)
                }
            }
            SliceDirection::Vertical => {
                if end.y < height as i32 / 2 {
                    Coord::new(end.x, -1)
                } else {
                    Coord::new(end.x, height as i32)
                }
            }
            SliceDirection::Diagonal => {
                if height <= 1 {
                    return Self::start_for(end, width, height, SliceDirection::Horizontal);
                }
                if end.x >= end.y {
                    Coord::new(width as i32, height as i32)
                } else {
                    Coord::new(-1, -1)
                }
            }
        }
    }

    fn color_for(coord: Coord, width: u16, height: u16) -> Color {
        let denom = (width as f64 + height as f64 - 2.0).max(1.0);
        let raw = (coord.x as f64 + coord.y as f64) / denom;
        let t = if raw < 0.0 {
            0.0
        } else if raw > 1.0 {
            1.0
        } else {
            raw
        };

        Gradient::new(vec![
            (0.0, Color::new(0, 255, 255)),
            (0.5, Color::new(255, 0, 255)),
            (1.0, Color::new(255, 255, 0)),
        ])
        .color_at(t)
    }

    fn parse_rows(input: &str) -> Vec<Vec<char>> {
        if input.trim().is_empty() {
            return vec!["TerminalTextEffects".chars().collect()];
        }

        let mut rows: Vec<Vec<char>> = input
            .lines()
            .map(|line| line.trim_end_matches('\r').chars().collect())
            .collect();

        if rows.iter().all(|r| r.is_empty()) {
            rows = vec!["TerminalTextEffects".chars().collect()];
        }

        rows
    }
}

impl Effect for Slice {
    fn name(&self) -> &str {
        "slice"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let rows = Self::parse_rows(input);
        let height = rows.len().max(1) as u16;
        let width = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1) as u16;

        let mut terminal = Terminal::new(width, height);
        let mut meta = Vec::new();
        let mut id = 0u32;

        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, &symbol) in row.iter().enumerate() {
                let end = Coord::new(col_idx as i32, row_idx as i32);
                let fg = Self::color_for(end, width, height);

                let mut character = EffectCharacter::new(id, end, symbol);
                character.color_pair = ColorPair::new(fg, Color::new(10, 10, 20));
                character.position = end;
                terminal.add_character(character);

                meta.push((Self::start_for(end, width, height, self.direction), end));
                id += 1;
            }
        }

        if meta.is_empty() {
            return vec![terminal.render_frame()];
        }

        let max_dist = meta
            .iter()
            .map(|(start, end)| {
                let dx = (end.x - start.x).abs() as f64;
                let dy = (end.y - start.y).abs() as f64;
                (dx * dx + dy * dy).sqrt()
            })
            .fold(0.0f64, |acc, value| acc.max(value));

        let total_frames = ((max_dist / self.speed).ceil() as usize)
            .max(2)
            .min(120);
        let easing: Box<dyn Easing> =
            get_easing(&self.easing_name).unwrap_or_else(|| Box::new(Linear));

        let mut frames = Vec::with_capacity(total_frames);

        for frame_idx in 0..total_frames {
            {
                let chars = terminal.get_characters_mut();
                for (idx, character) in chars.iter_mut().enumerate() {
                    let (start, end) = meta[idx];
                    if frame_idx == total_frames - 1 {
                        character.position = end;
                    } else {
                        let t = frame_idx as f64 / (total_frames - 1) as f64;
                        let eased = easing.ease(t);
                        let x = start.x as f64 + ((end.x - start.x) as f64) * eased;
                        let y = start.y as f64 + ((end.y - start.y) as f64) * eased;
                        character.position = Coord::new(x.round() as i32, y.round() as i32);
                    }
                }
            }
            frames.push(terminal.render_frame());
        }

        frames
    }
}
