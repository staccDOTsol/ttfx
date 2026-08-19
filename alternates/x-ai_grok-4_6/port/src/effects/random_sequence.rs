use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::graphics::{Color, Gradient};

pub struct RandomSequence;

impl RandomSequence {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for RandomSequence {
    fn name(&self) -> &str {
        "random_sequence"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let width = input
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = input.lines().count().max(1);
        let mut term = Terminal::from_input(input, width, height);

        let gradient = Gradient::new(
            vec![
                Color::rgb(0x3f, 0x37, 0xc9),
                Color::rgb(0x4c, 0xc9, 0xf0),
                Color::rgb(0xf7, 0x25, 0x85),
                Color::rgb(0xb5, 0x17, 0x9e),
            ],
            12,
        );
        let spectrum = gradient.colors();

        let n = term.get_characters().len();
        if n == 0 {
            return vec![term.get_formatted_output_string()];
        }

        let mut order: Vec<usize> = (0..n).collect();
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        for ch in term.get_characters() {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(ch.input_symbol as u64)
                .wrapping_add(ch.input_coord.column as u64)
                .wrapping_add((ch.input_coord.row as u64) << 16);
        }
        for i in (1..order.len()).rev() {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = (seed as usize) % (i + 1);
            order.swap(i, j);
        }

        let mut frames = Vec::new();
        let mut revealed = 0usize;
        let per_frame = (n / 24).max(1);

        loop {
            revealed = (revealed + per_frame).min(n);
            for &idx in &order[..revealed] {
                let id = term.get_characters()[idx].id;
                term.set_character_visibility(id, true);
                let color = spectrum[idx % spectrum.len()];
                if let Some(ch) = term.get_characters_mut().get_mut(idx) {
                    ch.animation.set_appearance(ch.input_symbol, Some(color));
                    ch.colors = Some(crate::utils::graphics::ColorPair {
                        fg: Some(color),
                        bg: None,
                    });
                }
            }
            term.step_all();

            let mut out = String::new();
            out.push_str("\x1b[H\x1b[2J");
            let mut grid: Vec<Vec<Option<(char, Color)>>> =
                vec![vec![None; width]; height];
            for ch in term.get_characters() {
                if !ch.is_visible {
                    continue;
                }
                let x = ch.current_coord.column;
                let y = ch.current_coord.row;
                if x < 0 || y < 0 {
                    continue;
                }
                let (x, y) = (x as usize, y as usize);
                if x >= width || y >= height {
                    continue;
                }
                let color = ch
                    .animation
                    .current_character_visual
                    .colors
                    .and_then(|p| p.fg)
                    .or_else(|| ch.colors.and_then(|p| p.fg))
                    .unwrap_or(Color::rgb(0x4c, 0xc9, 0xf0));
                grid[y][x] = Some((ch.animation.current_character_visual.symbol, color));
            }
            for row in &grid {
                for cell in row {
                    match cell {
                        Some((sym, c)) => {
                            out.push_str(&format!(
                                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                                c.r, c.g, c.b, sym
                            ));
                        }
                        None => out.push(' '),
                    }
                }
                out.push('\n');
            }
            frames.push(out);

            if revealed >= n {
                break;
            }
        }
        frames
    }
}
