use super::Effect;
use crate::utils::graphics::{Color, Gradient};

pub struct Laseretch;

impl Laseretch {
    pub fn new() -> Self {
        Laseretch
    }
}

impl Effect for Laseretch {
    fn name(&self) -> &str {
        "laseretch"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0).max(1);

        // Build a static grid of input symbols, padded with spaces.
        let mut symbols = vec![vec![' '; width]; height];
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                symbols[y][x] = ch;
            }
        }

        let laser_color = Color::new(255, 0, 0);
        let dim_color = Color::new(60, 60, 60);
        let flash_color = Color::new(255, 255, 255);

        // The final etched color for a character depends on its column.
        let gradient = Gradient::new(vec![
            (0.0, Color::new(0, 255, 255)),
            (0.5, Color::new(255, 0, 255)),
            (1.0, Color::new(255, 255, 0)),
        ]);

        let mut frames = Vec::new();
        let start_x: i32 = -1;
        let end_x: i32 = width as i32 + 1;

        for laser_x in start_x..=end_x {
            let mut cells = vec![vec![(Color::BLACK, Color::BLACK); width]; height];

            for y in 0..height {
                for x in 0..width {
                    let symbol = symbols[y][x];
                    let mut fg = if symbol != ' ' { dim_color } else { Color::BLACK };
                    let mut bg = Color::BLACK;

                    let x_i32 = x as i32;

                    if laser_x >= 0 && x_i32 == laser_x {
                        // The laser beam itself: a bright red vertical column.
                        bg = laser_color;
                        fg = if symbol != ' ' { flash_color } else { Color::BLACK };
                    } else if x_i32 <= laser_x {
                        // Once the beam passes, the character is etched with final colors.
                        if symbol != ' ' {
                            let t = if width > 1 {
                                x as f64 / (width - 1) as f64
                            } else {
                                0.0
                            };
                            fg = gradient.color_at(t);
                        }
                    }

                    cells[y][x] = (fg, bg);
                }
            }

            frames.push(render_frame(&symbols, &cells));
        }

        frames
    }
}

fn render_frame(symbols: &[Vec<char>], cells: &[Vec<(Color, Color)>]) -> String {
    let mut out = String::new();
    for y in 0..cells.len() {
        for x in 0..cells[y].len() {
            let (fg, bg) = cells[y][x];
            let ch = symbols[y][x];
            out.push_str(&format!(
                "\x1b[38;2;{};{};{};48;2;{};{};{}m{}\x1b[0m",
                fg.r, fg.g, fg.b, bg.r, bg.g, bg.b, ch
            ));
        }
        out.push('\n');
    }
    out
}
