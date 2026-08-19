use crate::engine::{EffectCharacter, Terminal};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

use super::Effect;

pub struct Spotlights;

impl Spotlights {
    pub fn new() -> Self {
        Spotlights
    }
}

impl Effect for Spotlights {
    fn name(&self) -> &str {
        "spotlights"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Parse input into lines. Preserve empty input as a single blank line.
        let raw_lines: Vec<&str> = if input.is_empty() {
            vec![" "]
        } else {
            input.lines().collect()
        };

        let height = raw_lines.len().max(1);
        let width = raw_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);

        let mut terminal = Terminal::new(width as u16, height as u16);

        let mut character_id = 0u32;
        for (row, line) in raw_lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let coord = Coord::new(col as i32, row as i32);
                let mut character = EffectCharacter::new(character_id, coord, ch);
                character.visible = false;
                terminal.add_character(character);
                character_id += 1;
            }
        }

        let paths = Self::generate_spotlight_paths(&terminal);
        let mut frames = Vec::new();

        const BEAM_RADIUS: f64 = 6.0;
        const SEGMENT_FRAMES: usize = 20;
        const TOTAL_FRAMES: usize = 180;

        let beam_gradient = Gradient::new(vec![
            (0.0, Color::new(255, 255, 255)),
            (0.3, Color::new(170, 210, 255)),
            (0.7, Color::new(80, 140, 255)),
            (1.0, Color::new(20, 60, 200)),
        ]);

        for frame_idx in 0..TOTAL_FRAMES {
            let mut centers = Vec::new();

            for path in &paths {
                let path_len = path.len().max(2);
                let cycle_len = path_len * SEGMENT_FRAMES;
                let pos_in_cycle = frame_idx % cycle_len;
                let segment = pos_in_cycle / SEGMENT_FRAMES;
                let progress = (pos_in_cycle % SEGMENT_FRAMES) as f64 / SEGMENT_FRAMES as f64;
                let eased = ease_in_out_quad(progress);

                let start = path[segment];
                let end = path[(segment + 1) % path_len];

                let x = start.x as f64 + ((end.x - start.x) as f64 * eased);
                let y = start.y as f64 + ((end.y - start.y) as f64 * eased);
                centers.push(Coord::new(x.round() as i32, y.round() as i32));
            }

            {
                let characters = terminal.get_characters_mut();
                for character in characters.iter_mut() {
                    let coord = character.position;
                    let mut best_distance = f64::INFINITY;

                    for center in &centers {
                        let distance = coord.distance(center);
                        if distance < best_distance {
                            best_distance = distance;
                        }
                    }

                    if best_distance <= BEAM_RADIUS {
                        let t = 1.0 - (best_distance / BEAM_RADIUS);
                        let color = beam_gradient.color_at(t);
                        character.visible = true;
                        character.color_pair = ColorPair::new(color, Color::BLACK);
                    } else {
                        character.visible = false;
                    }
                }
            }

            frames.push(terminal.render_frame());
        }

        // Final frame: all characters visible in white.
        {
            let characters = terminal.get_characters_mut();
            for character in characters.iter_mut() {
                character.visible = true;
                character.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
            }
        }
        frames.push(terminal.render_frame());

        frames
    }
}

impl Spotlights {
    fn generate_spotlight_paths(terminal: &Terminal) -> Vec<Vec<Coord>> {
        const SPOTLIGHT_COUNT: usize = 5;
        const WAYPOINT_COUNT: usize = 6;

        let width = terminal.canvas.width as i32;
        let height = terminal.canvas.height as i32;
        let mut rng = Rng::new(0x5eed_1234_5678_abcd);
        let mut paths = Vec::new();

        let min_distance = if width >= 8 && height >= 8 { 5.0 } else { 1.0 };

        for _ in 0..SPOTLIGHT_COUNT {
            let mut waypoints = Vec::new();

            let first_x = rng.uniform_int(0, (width - 1).max(0));
            let first_y = rng.uniform_int(0, (height - 1).max(0));
            let mut last = Coord::new(first_x, first_y);
            waypoints.push(last);

            for _ in 0..WAYPOINT_COUNT - 1 {
                let mut found = false;

                for _ in 0..200 {
                    let candidate = Coord::new(
                        rng.uniform_int(0, (width - 1).max(0)),
                        rng.uniform_int(0, (height - 1).max(0)),
                    );

                    if candidate != last && candidate.distance(&last) as f64 >= min_distance {
                        last = candidate;
                        waypoints.push(candidate);
                        found = true;
                        break;
                    }
                }

                if !found {
                    waypoints.push(last);
                }
            }

            if waypoints.len() < 2 {
                waypoints.push(Coord::new(width / 2, height / 2));
            }

            paths.push(waypoints);
        }

        paths
    }
}

fn ease_in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn uniform_int(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next_f64() * (max - min + 1) as f64).floor() as i32
    }
}
