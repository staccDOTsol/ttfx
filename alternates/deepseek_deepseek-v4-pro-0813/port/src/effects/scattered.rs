use super::Effect;
use crate::engine::{Canvas, EffectCharacter};
use crate::utils::easing::{get_easing, Easing};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Scattered;

impl Scattered {
    pub fn new() -> Self {
        Scattered
    }
}

impl Effect for Scattered {
    fn name(&self) -> &str {
        "scattered"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        scattered_frames(input)
    }
}

fn scattered_frames(input: &str) -> Vec<String> {
    let mut lines: Vec<&str> = input.lines().collect();
    if input.ends_with('\n') {
        lines.push("");
    }

    let height = lines.len().max(1);
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);

    let canvas = Canvas::new(width as u16, height as u16);
    let mut characters: Vec<EffectCharacter> = Vec::new();
    let mut starts: Vec<Coord> = Vec::new();
    let mut targets: Vec<Coord> = Vec::new();
    let mut char_steps: Vec<usize> = Vec::new();

    let mut rng = Rng::new(0x1234_5678_9ABC_DEF0);
    let easing = get_easing("cubic_out").expect("cubic_out easing must exist");

    let mut next_id: u32 = 0;
    for (row, line) in lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch.is_whitespace() {
                continue;
            }

            let target = Coord::new(col as i32, row as i32);
            let start = rng.next_coord(width as u16, height as u16);
            let steps = ((start.distance(&target) / 1.2).ceil() as usize).max(10);

            let mut character = EffectCharacter::new(next_id, start, ch);
            character.color_pair =
                ColorPair::new(character_color(target, width as u16), Color::BLACK);

            characters.push(character);
            starts.push(start);
            targets.push(target);
            char_steps.push(steps);
            next_id += 1;
        }
    }

    if characters.is_empty() {
        return vec![canvas.render(&characters)];
    }

    let total_steps = char_steps.iter().max().copied().unwrap().max(16);
    let mut frames = Vec::with_capacity(total_steps + 1);

    for frame_idx in 0..total_steps {
        for (i, character) in characters.iter_mut().enumerate() {
            let start = starts[i];
            let target = targets[i];
            let steps = char_steps[i];

            let t = if steps <= 1 {
                1.0
            } else {
                let raw = frame_idx as f64 / (steps - 1) as f64;
                raw.clamp(0.0, 1.0)
            };

            let eased = easing.ease(t);
            let x = start.x as f64 + (target.x - start.x) as f64 * eased;
            let y = start.y as f64 + (target.y - start.y) as f64 * eased;
            character.position = Coord::new(x.round() as i32, y.round() as i32);
        }

        frames.push(canvas.render(&characters));
    }

    // Ensure the final frame has every character exactly on its input coordinate.
    for (i, character) in characters.iter_mut().enumerate() {
        character.position = targets[i];
    }
    frames.push(canvas.render(&characters));

    frames
}

fn character_color(coord: Coord, width: u16) -> Color {
    let t = if width > 1 {
        coord.x as f64 / (width - 1) as f64
    } else {
        0.0
    };

    let gradient = Gradient::new(vec![
        (0.0, Color::new(0, 0, 255)),
        (0.25, Color::new(0, 255, 255)),
        (0.5, Color::new(0, 255, 0)),
        (0.75, Color::new(255, 255, 0)),
        (1.0, Color::new(255, 0, 0)),
    ]);

    gradient.color_at(t)
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        self.next_u64() as f64 / u64::MAX as f64
    }

    fn next_coord(&mut self, width: u16, height: u16) -> Coord {
        let x = (self.next_f64() * width as f64) as i32;
        let y = (self.next_f64() * height as f64) as i32;

        Coord::new(
            x.min(width as i32 - 1).max(0),
            y.min(height as i32 - 1).max(0),
        )
    }
}
