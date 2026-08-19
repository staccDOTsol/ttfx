use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Sweep;

impl Sweep {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sweep {
    fn default() -> Self {
        Self::new()
    }
}

fn sgr(pair: Option<ColorPair>, ch: char) -> String {
    match pair {
        Some(ColorPair {
            fg: Some(fg),
            bg: Some(bg),
        }) => format!(
            "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}\x1b[0m",
            fg.r, fg.g, fg.b, bg.r, bg.g, bg.b, ch
        ),
        Some(ColorPair {
            fg: Some(fg),
            bg: None,
        }) => format!("\x1b[38;2;{};{};{}m{}\x1b[0m", fg.r, fg.g, fg.b, ch),
        Some(ColorPair {
            fg: None,
            bg: Some(bg),
        }) => format!("\x1b[48;2;{};{};{}m{}\x1b[0m", bg.r, bg.g, bg.b, ch),
        _ => ch.to_string(),
    }
}

fn render(term: &Terminal, colors: &[Option<ColorPair>]) -> String {
    let w = term.canvas.width.max(1);
    let h = term.canvas.height.max(1);
    let mut grid = vec![vec![' '; w]; h];
    let mut cgrid: Vec<Vec<Option<ColorPair>>> = vec![vec![None; w]; h];
    for (ch, col) in term.get_characters().iter().zip(colors.iter()) {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 {
            let ux = x as usize;
            let uy = y as usize;
            if ux < w && uy < h {
                grid[uy][ux] = ch.animation.current_character_visual.symbol;
                cgrid[uy][ux] = *col;
            }
        }
    }
    let mut out = String::new();
    for y in 0..h {
        for x in 0..w {
            out.push_str(&sgr(cgrid[y][x], grid[y][x]));
        }
        out.push('\n');
    }
    out
}

impl Effect for Sweep {
    fn name(&self) -> &str {
        "sweep"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let gradient = Gradient::new(
            vec![
                Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
                Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
                Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
            ],
            8,
        );
        let spectrum = gradient.colors();

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        let mut positions: Vec<(Coord, char)> = term
            .get_characters()
            .iter()
            .map(|c| (c.input_coord, c.input_symbol))
            .collect();

        // sort by column then row — left-to-right sweep like the Python original
        let mut order: Vec<usize> = (0..ids.len()).collect();
        order.sort_by_key(|&i| (positions[i].0.column, positions[i].0.row));

        for id in &ids {
            term.set_character_visibility(*id, false);
        }

        let mut colors: Vec<Option<ColorPair>> = vec![None; ids.len()];
        let mut frames = Vec::new();

        // hold empty first frame
        frames.push(render(&term, &colors));

        let n = order.len();
        if n == 0 {
            return frames;
        }

        // sweep in: reveal by column groups
        let mut revealed = 0usize;
        while revealed < n {
            let col = positions[order[revealed]].0.column;
            while revealed < n && positions[order[revealed]].0.column == col {
                let idx = order[revealed];
                term.set_character_visibility(ids[idx], true);
                let step = (col as usize).min(spectrum.len().saturating_sub(1));
                let fg = spectrum[step % spectrum.len()];
                colors[idx] = Some(ColorPair {
                    fg: Some(fg),
                    bg: None,
                });
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == ids[idx]) {
                    ch.animation.set_appearance(positions[idx].1, Some(fg));
                    let scn = ch.animation.new_scene("sweep");
                    for (i, c) in spectrum.iter().enumerate() {
                        scn.add_frame(positions[idx].1, 2, Some(ColorPair {
                            fg: Some(*c),
                            bg: None,
                        }));
                        let _ = i;
                    }
                    ch.animation.activate_scene("sweep");
                }
                revealed += 1;
            }
            term.step_all();
            for (i, ch) in term.get_characters().iter().enumerate() {
                if ch.is_visible {
                    if let Some(vis) = ch.animation.current_character_visual.colors {
                        colors[i] = Some(vis);
                    }
                }
            }
            frames.push(render(&term, &colors));
        }

        // settle: run remaining animation frames so gradient finishes
        for _ in 0..12 {
            term.step_all();
            for (i, ch) in term.get_characters().iter().enumerate() {
                if ch.is_visible {
                    if let Some(vis) = ch.animation.current_character_visual.colors {
                        colors[i] = Some(vis);
                    } else if let Some(last) = spectrum.last() {
                        colors[i] = Some(ColorPair {
                            fg: Some(*last),
                            bg: None,
                        });
                    }
                }
            }
            frames.push(render(&term, &colors));
        }

        let _ = positions;
        frames
    }
}
