use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};
use std::f64::consts::PI;

struct CharState {
    id: u32,
    input_coord: Coord,
    ring_target: Coord,
    base_angle: f64,
    ring_radius: f64,
}

pub struct Rings;

impl Rings {
    pub fn new() -> Self {
        Rings
    }
}

impl Effect for Rings {
    fn name(&self) -> &str {
        "rings"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let (char_inputs, width, height) = parse_input(input);
        let width = width.max(1);
        let height = height.max(1);

        let mut terminal = Terminal::new(width, height);
        let center = Coord::new((width as i32) / 2, (height as i32) / 2);

        let gradient = Gradient::new(vec![
            (0.0, Color::new(0, 255, 255)),
            (0.5, Color::new(0, 128, 255)),
            (1.0, Color::new(255, 0, 255)),
        ]);

        let ring_coords = ring_coords_for_chars(center, char_inputs.len(), width, height, 3);
        let max_dist = (width.max(height) as f64 / 2.0).max(1.0);

        let mut states: Vec<CharState> = Vec::new();

        for (i, (ch, input_coord)) in char_inputs.into_iter().enumerate() {
            let ring_target = if ring_coords.is_empty() {
                input_coord
            } else {
                ring_coords[i % ring_coords.len()]
            };

            let dx = (input_coord.x - center.x) as f64;
            let dy = (input_coord.y - center.y) as f64;
            let dist = (dx * dx + dy * dy).sqrt();
            let color = gradient.color_at((dist / max_dist).min(1.0));

            let mut character = EffectCharacter::new(i as u32, input_coord, ch);
            character.color_pair = ColorPair::new(color, Color::BLACK);
            character.visible = true;
            terminal.add_character(character);

            let ring_radius = ring_target.distance(&center);
            let base_angle =
                (ring_target.y as f64 - center.y as f64).atan2(ring_target.x as f64 - center.x as f64);

            states.push(CharState {
                id: i as u32,
                input_coord,
                ring_target,
                base_angle,
                ring_radius,
            });
        }

        let expansion_frames = 30usize;
        let rotation_frames = 60usize;
        let return_frames = 30usize;
        let total_frames = expansion_frames + rotation_frames + return_frames;

        let mut frames = Vec::with_capacity(total_frames);

        for frame_idx in 0..total_frames {
            for state in &states {
                let pos = if frame_idx < expansion_frames {
                    let t = ease_in_out_sine(frame_idx as f64 / expansion_frames as f64);
                    lerp_coord(center, state.ring_target, t)
                } else if frame_idx < expansion_frames + rotation_frames {
                    let rot_t =
                        (frame_idx - expansion_frames) as f64 / rotation_frames as f64;
                    let angle = state.base_angle + rot_t * 2.0 * PI;
                    let raw = Coord::new(
                        center.x + (state.ring_radius * angle.cos()).round() as i32,
                        center.y + (state.ring_radius * angle.sin()).round() as i32,
                    );
                    clamp_coord(raw, width, height)
                } else {
                    let t = (frame_idx - expansion_frames - rotation_frames) as f64
                        / return_frames as f64;
                    lerp_coord(state.ring_target, state.input_coord, ease_out_quad(t))
                };

                if let Some(character) = terminal.get_character_mut(state.id) {
                    character.position = pos;
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}

fn parse_input(input: &str) -> (Vec<(char, Coord)>, u16, u16) {
    let lines: Vec<&str> = input.lines().collect();
    if lines.is_empty() {
        return (Vec::new(), 1, 1);
    }

    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
        .max(1) as u16;
    let height = lines.len().max(1) as u16;

    let mut chars = Vec::new();
    for (y, line) in lines.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch != ' ' && !ch.is_control() {
                chars.push((ch, Coord::new(x as i32, y as i32)));
            }
        }
    }

    (chars, width, height)
}

fn ring_coords_for_chars(
    center: Coord,
    needed: usize,
    width: u16,
    height: u16,
    ring_gap: i32,
) -> Vec<Coord> {
    let mut coords = Vec::new();
    if needed == 0 {
        return coords;
    }

    let max_radius = (width.max(height) as i32).max(1);
    let mut radius = 1i32;

    while coords.len() < needed && radius <= max_radius {
        let n_points = ((7 * radius).max(8)) as usize;

        for i in 0..n_points {
            let angle = 2.0 * PI * i as f64 / n_points as f64;
            let raw = Coord::new(
                (center.x as f64 + radius as f64 * angle.cos()).round() as i32,
                (center.y as f64 + radius as f64 * angle.sin()).round() as i32,
            );
            coords.push(clamp_coord(raw, width, height));
        }

        radius += ring_gap;
    }

    if coords.is_empty() {
        coords.push(center);
    }

    let mut idx = 0;
    while coords.len() < needed {
        coords.push(coords[idx % coords.len()]);
        idx += 1;
    }

    coords
}

fn clamp_coord(coord: Coord, width: u16, height: u16) -> Coord {
    let max_x = (width as i32).saturating_sub(1).max(0);
    let max_y = (height as i32).saturating_sub(1).max(0);

    Coord::new(coord.x.clamp(0, max_x), coord.y.clamp(0, max_y))
}

fn lerp_coord(a: Coord, b: Coord, t: f64) -> Coord {
    let x = a.x as f64 + (b.x as f64 - a.x as f64) * t;
    let y = a.y as f64 + (b.y as f64 - a.y as f64) * t;
    Coord::new(x.round() as i32, y.round() as i32)
}

fn ease_in_out_sine(t: f64) -> f64 {
    0.5 * (1.0 - (t * PI).cos())
}

fn ease_out_quad(t: f64) -> f64 {
    t * (2.0 - t)
}
