use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Highlight {
    highlight_color: Color,
    final_color: Color,
    sweep_steps: usize,
}

impl Highlight {
    pub fn new() -> Self {
        Self {
            highlight_color: Color::from_hex("ffff00").unwrap_or(Color::rgb(255, 255, 0)),
            final_color: Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
            sweep_steps: 8,
        }
    }
}

impl Default for Highlight {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Highlight {
    fn name(&self) -> &str {
        "highlight"
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

        let highlight_grad = Gradient::new(
            vec![
                Color::rgb(40, 40, 40),
                self.highlight_color,
                self.final_color,
            ],
            self.sweep_steps,
        );
        let grad_colors = highlight_grad.colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        for ch in term.get_characters_mut() {
            let scn = ch.animation.new_scene("highlight");
            for c in &grad_colors {
                scn.add_frame(
                    ch.input_symbol,
                    2,
                    Some(ColorPair {
                        fg: Some(*c),
                        bg: None,
                    }),
                );
            }
            scn.add_frame(
                ch.input_symbol,
                4,
                Some(ColorPair {
                    fg: Some(self.final_color),
                    bg: None,
                }),
            );
        }

        let mut frames = Vec::new();
        let n = ids.len();
        if n == 0 {
            return vec![term.get_formatted_output_string()];
        }

        let mut activated = vec![false; n];
        let mut lead: i32 = -2;
        let total_ticks = (n as i32) + (grad_colors.len() as i32 * 3) + 6;

        for _ in 0..total_ticks {
            lead += 1;
            for (i, id) in ids.iter().enumerate() {
                if (i as i32) <= lead && !activated[i] {
                    if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                        ch.animation.activate_scene("highlight");
                    }
                    activated[i] = true;
                }
            }
            term.step_all();
            frames.push(render_ansi(&term));
        }
        frames
    }
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<(char, Option<ColorPair>)>> =
        vec![vec![(' ', None); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 {
            let ux = x as usize;
            let uy = y as usize;
            if ux < w && uy < h {
                let vis = &ch.animation.current_character_visual;
                let colors = vis.colors.or(ch.colors);
                grid[uy][ux] = (vis.symbol, colors);
            }
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, colors) in row {
            if let Some(cp) = colors {
                if let Some(fg) = cp.fg {
                    out.push_str(&format!("\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b));
                }
                if let Some(bg) = cp.bg {
                    out.push_str(&format!("\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b));
                }
                out.push(sym);
                out.push_str("\x1b[0m");
            } else {
                out.push(sym);
            }
        }
        out.push('\n');
    }
    out
}
