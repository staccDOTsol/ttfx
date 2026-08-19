use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, Gradient};

pub struct Wipe;

impl Wipe {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Wipe {
    fn name(&self) -> &str {
        "wipe"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = if input.is_empty() {
            vec![""]
        } else {
            input.lines().collect()
        };
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let gradient = Gradient::new(
            vec![
                Color::rgb(0x21, 0x3a, 0x8f),
                Color::rgb(0x00, 0xd4, 0xff),
                Color::rgb(0xff, 0xff, 0xff),
                Color::rgb(0xff, 0xb3, 0x47),
            ],
            8,
        );
        let spectrum = gradient.colors();

        let mut by_col: Vec<(i32, Vec<CharacterId>)> = Vec::new();
        {
            let mut buckets: std::collections::BTreeMap<i32, Vec<CharacterId>> =
                std::collections::BTreeMap::new();
            for ch in term.get_characters() {
                buckets.entry(ch.input_coord.column).or_default().push(ch.id);
            }
            for (col, ids) in buckets {
                by_col.push((col, ids));
            }
        }

        let mut frames = Vec::new();
        let hold = 3usize;
        let mut revealed: Vec<(CharacterId, usize)> = Vec::new();

        for (_col, ids) in &by_col {
            for id in ids {
                term.set_character_visibility(*id, true);
                revealed.push((*id, 0));
            }
            for _ in 0..hold {
                step_wipe(&mut term, &mut revealed, &spectrum);
                frames.push(render_ansi(&term, &spectrum, &revealed));
            }
        }
        for _ in 0..12 {
            step_wipe(&mut term, &mut revealed, &spectrum);
            frames.push(render_ansi(&term, &spectrum, &revealed));
        }
        if frames.is_empty() {
            frames.push(render_ansi(&term, &spectrum, &revealed));
        }
        frames
    }
}

fn step_wipe(term: &mut Terminal, revealed: &mut [(CharacterId, usize)], spectrum: &[Color]) {
    for (_id, age) in revealed.iter_mut() {
        *age = (*age + 1).min(spectrum.len().saturating_sub(1).max(1) + 4);
    }
    term.step_all();
}

fn sgr(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

fn render_ansi(
    term: &Terminal,
    spectrum: &[Color],
    revealed: &[(CharacterId, usize)],
) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid = vec![vec![(' ', None::<Color>); w]; h];
    let age_of = |id: CharacterId| {
        revealed
            .iter()
            .find(|(i, _)| *i == id)
            .map(|(_, a)| *a)
            .unwrap_or(0)
    };
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x < 0 || y < 0 {
            continue;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= w || uy >= h {
            continue;
        }
        let age = age_of(ch.id);
        let color = if spectrum.is_empty() {
            Color::rgb(255, 255, 255)
        } else {
            spectrum[age.min(spectrum.len() - 1)]
        };
        let sym = ch.animation.current_character_visual.symbol;
        grid[uy][ux] = (sym, Some(color));
    }
    let mut out = String::new();
    for row in grid {
        for (sym, col) in row {
            if let Some(c) = col {
                out.push_str(&sgr(c));
                out.push(sym);
                out.push_str("\x1b[0m");
            } else {
                out.push(' ');
            }
        }
        out.push('\n');
    }
    let _ = Coord::new(0, 0);
    out
}
