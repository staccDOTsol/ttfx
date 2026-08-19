use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::easing::{get_easing, Easing};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

use super::Effect;

#[derive(Clone, Copy)]
enum ExpandDirection {
    Vertical,
    Horizontal,
}

struct MiddleoutConfig {
    expand_direction: ExpandDirection,
    center_movement_speed: f64,
    full_movement_speed: f64,
    center_easing: String,
    full_easing: String,
}

impl Default for MiddleoutConfig {
    fn default() -> Self {
        Self {
            expand_direction: ExpandDirection::Vertical,
            center_movement_speed: 0.5,
            full_movement_speed: 0.5,
            center_easing: "in_out_sine".to_string(),
            full_easing: "in_out_quad".to_string(),
        }
    }
}

struct CharAnim {
    id: u32,
    input: Coord,
    center: Coord,
}

pub struct Middleout {
    config: MiddleoutConfig,
}

impl Middleout {
    pub fn new() -> Self {
        Self {
            config: MiddleoutConfig::default(),
        }
    }

    fn ease_progress(&self, name: &str, t: f64) -> f64 {
        match get_easing(name) {
            Some(e) => e.ease(t),
            None => t,
        }
    }

    fn render_frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        if lines.is_empty() {
            return Vec::new();
        }

        let width_u16 = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1) as u16;
        let height_u16 = lines.len().max(1) as u16;

        let mut terminal = Terminal::new(width_u16, height_u16);
        let mut anims = Vec::new();
        let mut next_id = 0u32;

        let gradient = Gradient::new(vec![
            (0.0, Color::new(255, 0, 0)),
            (0.5, Color::new(0, 255, 0)),
            (1.0, Color::new(0, 0, 255)),
        ]);

        let center_row = height_u16 / 2;
        let center_col = width_u16 / 2;

        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let input_coord = Coord::new(x as i32, y as i32);
                let center_coord = match self.config.expand_direction {
                    ExpandDirection::Vertical => Coord::new(input_coord.x, center_row as i32),
                    ExpandDirection::Horizontal => Coord::new(center_col as i32, input_coord.y),
                };

                let color_t = match self.config.expand_direction {
                    ExpandDirection::Vertical => input_coord.x as f64 / width_u16 as f64,
                    ExpandDirection::Horizontal => input_coord.y as f64 / height_u16 as f64,
                };
                let fg = gradient.color_at(color_t);

                let mut character = EffectCharacter::new(next_id, input_coord, ch);
                character.color_pair = ColorPair::new(fg, Color::BLACK);
                terminal.add_character(character);

                anims.push(CharAnim {
                    id: next_id,
                    input: input_coord,
                    center: center_coord,
                });

                next_id += 1;
            }
        }

        let max_distance = anims
            .iter()
            .map(|a| a.input.distance(&a.center))
            .fold(0.0f64, f64::max)
            .max(1.0);

        let center_steps = ((max_distance / (self.config.center_movement_speed * 0.25)) as usize)
            .clamp(2, 100);
        let full_steps = ((max_distance / (self.config.full_movement_speed * 0.25)) as usize)
            .clamp(2, 100);

        let mut frames = Vec::new();

        // Inward phase: input -> center line
        for i in 0..center_steps {
            let t = i as f64 / (center_steps - 1) as f64;
            let eased = self.ease_progress(&self.config.center_easing, t);

            for anim in &anims {
                let pos = lerp_coord(anim.input, anim.center, eased);
                if let Some(character) = terminal.get_character_mut(anim.id) {
                    character.position = pos;
                }
            }

            frames.push(terminal.render_frame());
        }

        // Outward phase: center line -> input (skip t=0 to avoid duplicate center frame)
        for i in 1..full_steps {
            let t = i as f64 / (full_steps - 1) as f64;
            let eased = self.ease_progress(&self.config.full_easing, t);

            for anim in &anims {
                let pos = lerp_coord(anim.center, anim.input, eased);
                if let Some(character) = terminal.get_character_mut(anim.id) {
                    character.position = pos;
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
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
        self.render_frames(input)
    }
}

fn lerp_coord(a: Coord, b: Coord, t: f64) -> Coord {
    let x = a.x as f64 + (b.x as f64 - a.x as f64) * t;
    let y = a.y as f64 + (b.y as f64 - a.y as f64) * t;
    Coord::new(py_round(x), py_round(y))
}

/// Python's `round` uses banker's rounding (half-to-even).
fn py_round(value: f64) -> i32 {
    let floor = value.floor();
    let diff = value - floor;
    if diff == 0.5 {
        if (floor as i64).rem_euclid(2) == 0 {
            floor as i32
        } else {
            (floor + 1.0) as i32
        }
    } else {
        value.round() as i32
    }
}
