
use super::Effect;
use crate::engine::{EffectCharacter, Terminal};
use crate::utils::easing::{get_easing, Easing, Linear};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};
use std::f64::consts::PI;
use std::cmp::min;

/// Reproduce the `beams` effect: lines radiating from the canvas center,
/// drawing characters outward along each beam with a tail gradient.
pub struct Beams;

impl Beams {
    pub fn new() -> Self {
        Beams
    }
}

impl Effect for Beams {
    fn name(&self) -> &str {
        "beams"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Parse input into a grid of characters
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len() as u16;
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        if width == 0 || height == 0 {
            return vec!["".to_string()];
        }

        // Terminal holds all characters and renders the canvas
        let mut terminal = Terminal::new(width, height);
        let mut id = 0;
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                let character = EffectCharacter::new(id, coord, ch);
                terminal.add_character(character);
                id += 1;
            }
        }

        let center = Coord::new((width / 2) as i32, (height / 2) as i32);

        // Default configuration matching terminaltexteffects beams
        let beam_count = 8;
        let beam_delay = 2;          // frames between beam starts
        let beam_speed = 1.2;        // characters per frame
        let tail_length = 5;         // number of tail characters with gradient
        let beam_color = Color::new(220, 220, 255);   // near-white head
        let tail_color = Color::new(100, 100, 220);   // blue tail
        let bg_color = Color::BLACK;
        let hold_frames = 10;        // frames after all beams complete

        let character_count = terminal.characters.len();
        if character_count == 0 {
            return vec!["".to_string()];
        }

        // Avoid more beams than characters
        let beam_count = beam_count.min(character_count).max(1);

        // Assign characters to beams in round-robin fashion
        // (mirrors the Python effect's even distribution)
        let mut beam_assignments: Vec<Vec<usize>> = vec![Vec::new(); beam_count];
        for (idx, _) in (0..character_count).enumerate() {
            beam_assignments[idx % beam_count].push(idx);
        }

        // Generate coordinate line for each beam
        let mut beam_coords: Vec<Vec<Coord>> = Vec::with_capacity(beam_count);
        for i in 0..beam_count {
            let angle = (i as f64) * (2.0 * PI) / (beam_count as f64);
            let coords = line_coords(center, angle, width as f64, height as f64);
            beam_coords.push(coords);
        }

        // Beam length is the minimum of available coords and assigned characters
        let mut beam_lengths = Vec::with_capacity(beam_count);
        for i in 0..beam_count {
            let max_len = min(beam_coords[i].len(), beam_assignments[i].len());
            beam_lengths.push(max_len);
        }

        // Calculate total animation frames
        let mut max_completion = 0.0_f64;
        for i in 0..beam_count {
            let start = i as f64 * beam_delay as f64;
            let total = beam_lengths[i] as f64 / beam_speed;
            let completion = start + total;
            if completion > max_completion {
                max_completion = completion;
            }
        }
        let max_frame = (max_completion.ceil() as usize) + hold_frames;

        // Easing function for beam head movement (cubic_out like Python)
        let easing = get_easing("cubic_out")
            .unwrap_or_else(|| Box::new(Linear));

        let mut frames: Vec<String> = Vec::with_capacity(max_frame + 1);

        for frame_idx in 0..=max_frame {
            for i in 0..beam_count {
                let start_frame = i as f64 * beam_delay as f64;
                let elapsed = frame_idx as f64 - start_frame;
                if elapsed < 0.0 {
                    continue;
                }

                let total_frames = beam_lengths[i] as f64 / beam_speed;
                if total_frames > 0.0 {
                    let fraction = (elapsed / total_frames).clamp(0.0, 1.0);
                    let eased = easing.ease(fraction);
                    let progress = (eased * beam_lengths[i] as f64).round() as usize;
                    let progress = progress.min(beam_lengths[i]);

                    // Update all characters belonging to this beam
                    for char_idx_in_beam in 0..beam_assignments[i].len() {
                        let char_global_idx = beam_assignments[i][char_idx_in_beam];
                        let character = &mut terminal.characters[char_global_idx];

                        if char_idx_in_beam < progress {
                            // Place character at its coordinate on the beam
                            if char_idx_in_beam < beam_coords[i].len() {
                                character.position = beam_coords[i][char_idx_in_beam];
                                character.visible = true;

                                // Color with tail gradient
                                if progress > 0 {
                                    let head_idx = progress - 1;
                                    let dist = head_idx - char_idx_in_beam;
                                    if dist < tail_length {
                                        let ratio = (tail_length - dist) as f64 / tail_length as f64;
                                        let fg = interpolate_color(tail_color, beam_color, ratio);
                                        character.color_pair = ColorPair::new(fg, bg_color);
                                    } else {
                                        character.color_pair = ColorPair::new(beam_color, bg_color);
                                    }
                                } else {
                                    character.color_pair = ColorPair::new(beam_color, bg_color);
                                }
                            }
                        } else {
                            // Not yet placed, keep hidden
                            character.visible = false;
                        }
                    }
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}

/// Generate integer coordinates along a ray starting at `center` with angle `angle`,
/// stopping at the canvas boundary.
fn line_coords(center: Coord, angle: f64, width: f64, height: f64) -> Vec<Coord> {
    let mut coords = Vec::new();
    let step = 0.5;
    let max_dist = (width * width + height * height).sqrt() + 1.0;
    let cos = angle.cos();
    let sin = angle.sin();
    let mut t = 0.0;

    while t <= max_dist {
        let x = center.x as f64 + t * cos;
        let y = center.y as f64 + t * sin;
        let coord = Coord::new(x.round() as i32, y.round() as i32);

        if coord.x < 0 || coord.x >= width as i32 || coord.y < 0 || coord.y >= height as i32 {
            break;
        }

        if coords.last() != Some(&coord) {
            coords.push(coord);
        }
        t += step;
    }

    coords
}

/// Linearly interpolate between two colors.
fn interpolate_color(c1: Color, c2: Color, t: f64) -> Color {
    let r = (c1.r as f64 + (c2.r as f64 - c1.r as f64) * t).round() as u8;
    let g = (c1.g as f64 + (c2.g as f64 - c1.g as f64) * t).round() as u8;
    let b = (c1.b as f64 + (c2.b as f64 - c1.b as f64) * t).round() as u8;
    Color::new(r, g, b)
}
