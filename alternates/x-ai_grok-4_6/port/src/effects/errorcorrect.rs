use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, Gradient};

pub struct Errorcorrect;

impl Errorcorrect {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Errorcorrect {
    fn name(&self) -> &str {
        "errorcorrect"
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

        let error_color = Color::from_hex("bb0000").unwrap_or(Color::rgb(187, 0, 0));
        let correct_color = Color::from_hex("00bb00").unwrap_or(Color::rgb(0, 187, 0));
        let wipe = Gradient::new(vec![error_color, correct_color], 8).colors();

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        // Pairwise swap current positions (simple error-correct setup).
        {
            let chars = term.get_characters_mut();
            let n = chars.len();
            let mut i = 0;
            while i + 1 < n {
                let a = chars[i].current_coord;
                let b = chars[i + 1].current_coord;
                chars[i].current_coord = b;
                chars[i].motion.current_coord = b;
                chars[i + 1].current_coord = a;
                chars[i + 1].motion.current_coord = a;
                i += 2;
            }
        }

        let start_coords: Vec<Coord> = term
            .get_characters()
            .iter()
            .map(|c| c.current_coord)
            .collect();
        let dest_coords: Vec<Coord> = term
            .get_characters()
            .iter()
            .map(|c| c.input_coord)
            .collect();
        let symbols: Vec<char> = term
            .get_characters()
            .iter()
            .map(|c| c.input_symbol)
            .collect();

        let steps = 18usize;
        let mut out = Vec::with_capacity(steps + 4);

        for s in 0..steps {
            let t = s as f64 / (steps - 1).max(1) as f64;
            let color = wipe[(s * wipe.len() / steps).min(wipe.len().saturating_sub(1))];
            let mut grid = vec![vec![' '; width]; height];
            let mut paint = vec![vec![None; width]; height];
            for (i, &sym) in symbols.iter().enumerate() {
                let a = start_coords[i];
                let b = dest_coords[i];
                let col = (a.column as f64 + (b.column - a.column) as f64 * t).round() as i32;
                let row = (a.row as f64 + (b.row - a.row) as f64 * t).round() as i32;
                if col >= 0 && row >= 0 {
                    let x = col as usize;
                    let y = row as usize;
                    if x < width && y < height {
                        grid[y][x] = sym;
                        paint[y][x] = Some(color);
                    }
                }
            }
            out.push(render_ansi(&grid, &paint));
        }

        // Hold corrected text in green.
        let mut grid = vec![vec![' '; width]; height];
        let mut paint = vec![vec![None; width]; height];
        for (i, &sym) in symbols.iter().enumerate() {
            let c = dest_coords[i];
            if c.column >= 0 && c.row >= 0 {
                let x = c.column as usize;
                let y = c.row as usize;
                if x < width && y < height {
                    grid[y][x] = sym;
                    paint[y][x] = Some(correct_color);
                }
            }
        }
        let hold = render_ansi(&grid, &paint);
        for _ in 0..4 {
            out.push(hold.clone());
        }
        out
    }
}

fn render_ansi(grid: &[Vec<char>], paint: &[Vec<Option<Color>>]) -> String {
    let mut s = String::new();
    for (y, row) in grid.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            if let Some(c) = paint[y][x] {
                s.push_str(&format!(
                    "\x1b[38;2;{};{};{}m{}\x1b[0m",
                    c.r, c.g, c.b, ch
                ));
            } else {
                s.push(*ch);
            }
        }
        s.push('\n');
    }
    s
}
