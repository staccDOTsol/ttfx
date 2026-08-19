use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Burn;

impl Burn {
    pub fn new() -> Self {
        Self
    }
}

fn sgr(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

impl Effect for Burn {
    fn name(&self) -> &str {
        "burn"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let flame = Gradient::new(
            vec![
                Color::rgb(0xff, 0xf7, 0xa1),
                Color::rgb(0xff, 0xc1, 0x07),
                Color::rgb(0xff, 0x6d, 0x00),
                Color::rgb(0xd5, 0x00, 0x00),
                Color::rgb(0x21, 0x21, 0x21),
            ],
            12,
        );
        let palette = flame.colors();
        let ember = Color::rgb(0x42, 0x42, 0x42);

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        let burn_syms = ['#', '*', '+', '.', ' '];
        let mut out = Vec::new();
        let waves = (height + palette.len() + 4).max(16);

        for wave in 0..waves {
            term.step_all();
            let mut frame = String::new();
            let chars: Vec<_> = term
                .get_characters()
                .iter()
                .map(|c| (c.id, c.input_coord, c.input_symbol, c.current_coord))
                .collect();

            let mut grid: Vec<Vec<char>> = vec![vec![' '; width]; height];
            let mut color_grid: Vec<Vec<Option<Color>>> = vec![vec![None; width]; height];

            for (_id, input, symbol, _cur) in &chars {
                let col = input.column as usize;
                let row = input.row as usize;
                if row >= height || col >= width {
                    continue;
                }
                let progress = wave as i32 - (height as i32 - 1 - input.row);
                if progress < 0 {
                    grid[row][col] = *symbol;
                    color_grid[row][col] = Some(palette[0]);
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *_id) {
                        ch.animation.set_appearance(*symbol, Some(palette[0]));
                        ch.colors = Some(ColorPair {
                            fg: Some(palette[0]),
                            bg: None,
                        });
                    }
                } else {
                    let idx = (progress as usize).min(palette.len().saturating_sub(1));
                    let color = if idx + 1 >= palette.len() {
                        ember
                    } else {
                        palette[idx]
                    };
                    let si = (progress as usize).min(burn_syms.len() - 1);
                    let sym = if si + 1 >= burn_syms.len() {
                        ' '
                    } else {
                        burn_syms[si]
                    };
                    grid[row][col] = if sym == ' ' { ' ' } else { if si == 0 { *symbol } else { sym } };
                    color_grid[row][col] = Some(color);
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *_id) {
                        ch.animation.set_appearance(grid[row][col], Some(color));
                        ch.colors = Some(ColorPair {
                            fg: Some(color),
                            bg: None,
                        });
                    }
                }
            }

            for y in 0..height {
                for x in 0..width {
                    if let Some(c) = color_grid[y][x] {
                        frame.push_str(&sgr(c));
                        frame.push(grid[y][x]);
                        frame.push_str("\x1b[0m");
                    } else {
                        frame.push(' ');
                    }
                }
                frame.push('\n');
            }
            out.push(frame);
        }
        out
    }
}
