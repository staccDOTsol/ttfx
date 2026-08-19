use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Synthgrid;

impl Synthgrid {
    pub fn new() -> Self {
        Synthgrid
    }
}

impl Effect for Synthgrid {
    fn name(&self) -> &str {
        "synthgrid"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        synthgrid_frames(input)
    }
}

fn synthgrid_frames(input: &str) -> Vec<String> {
    let width: u16 = 80;
    let height: u16 = 24;
    let mut frames = Vec::new();

    let input_chars: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();

    let grid_gradient = Gradient::new(vec![
        (0.0, Color::new(255, 0, 255)), // magenta
        (0.5, Color::new(0, 255, 255)), // cyan
        (1.0, Color::new(255, 0, 255)), // magenta
    ]);

    let frame_count = 12u16;
    for frame_idx in 0..frame_count {
        let mut terminal = Terminal::new(width, height);
        let mut id = 0u32;

        // Retro sun centered above the horizon.
        let sun_center = Coord::new(40, 5);
        let sun_radius = 5.0;
        for y in 0..height {
            for x in 0..width {
                let coord = Coord::new(x as i32, y as i32);
                let dist = coord.distance(&sun_center);
                if dist <= sun_radius {
                    let t = dist / sun_radius;
                    let color = grid_gradient.color_at(t);
                    let mut ec = EffectCharacter::new(id, coord, '█');
                    ec.color_pair = ColorPair::new(color, Color::BLACK);
                    ec.bold = true;
                    terminal.add_character(ec);
                    id += 1;
                }
            }
        }

        // Horizontal grid lines, scrolled by frame index.
        let scroll = (frame_idx % 4) as i32;
        let mut y = 8 + scroll;
        while y < height as i32 {
            let t = (y as f64 - 8.0) / (height as f64 - 8.0);
            let line_color = grid_gradient.color_at(t);
            for x in 0..width as i32 {
                let coord = Coord::new(x, y);
                let mut ec = EffectCharacter::new(id, coord, '█');
                ec.color_pair = ColorPair::new(line_color, Color::BLACK);
                ec.bold = true;
                terminal.add_character(ec);
                id += 1;
            }
            y += 4;
        }

        // Vertical converging grid lines (approximate perspective).
        for x in (0..width as i32).step_by(10) {
            for y in 8..height as i32 {
                let t = (y as f64 - 8.0) / (height as f64 - 8.0);
                let line_color = grid_gradient.color_at(t);
                let coord = Coord::new(x, y);
                let mut ec = EffectCharacter::new(id, coord, '█');
                ec.color_pair = ColorPair::new(line_color, Color::BLACK);
                ec.dim = true;
                terminal.add_character(ec);
                id += 1;
            }
        }

        // Place input characters on/above the grid.
        for (i, &ch) in input_chars.iter().enumerate() {
            let col = (i % 70 + 5) as i32;
            let row = 8 + (i / 70) as i32;
            if row >= height as i32 {
                break;
            }
            let coord = Coord::new(col, row);
            let t = (col as f64 / width as f64 + frame_idx as f64 * 0.08) % 1.0;
            let color = grid_gradient.color_at(t);
            let mut ec = EffectCharacter::new(id, coord, ch);
            ec.color_pair = ColorPair::new(color, Color::BLACK);
            ec.bold = true;
            terminal.add_character(ec);
            id += 1;
        }

        frames.push(terminal.render_frame());
    }

    frames
}
