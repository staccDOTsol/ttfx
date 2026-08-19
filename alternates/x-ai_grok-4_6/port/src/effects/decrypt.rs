use super::Effect;
use crate::engine::character::{CharacterId, EffectCharacter};
use crate::engine::terminal::Terminal;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const CIPHER: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    'A', 'B', 'C', 'D', 'E', 'F', '@', '#', '$', '%', '&', '*', '+', '=', '?',
];

pub struct Decrypt;

impl Decrypt {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Decrypt {
    fn name(&self) -> &str {
        "decrypt"
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
        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        if ids.is_empty() {
            return vec![String::new()];
        }

        let cipher_stops = vec![
            Color::from_hex("00ff00").unwrap_or(Color::rgb(0, 255, 0)),
            Color::from_hex("88ff88").unwrap_or(Color::rgb(136, 255, 136)),
            Color::from_hex("003300").unwrap_or(Color::rgb(0, 51, 0)),
        ];
        let cipher_grad = Gradient::new(cipher_stops, 12);
        let cipher_colors = cipher_grad.colors();

        let final_stops = vec![
            Color::from_hex("00aa00").unwrap_or(Color::rgb(0, 170, 0)),
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let final_grad = Gradient::new(final_stops, 8);
        let final_colors = final_grad.colors();

        for ch in term.get_characters_mut() {
            let id_n = ch.id.0 as usize;
            let mut scramble = ch.animation.new_scene("scramble");
            scramble.is_looping = true;
            for k in 0..16 {
                let sym = CIPHER[(id_n + k * 7) % CIPHER.len()];
                let col = cipher_colors[(id_n + k) % cipher_colors.len()];
                scramble.add_frame(
                    sym,
                    1,
                    Some(ColorPair {
                        fg: Some(col),
                        bg: None,
                    }),
                );
            }
            let mut settle = ch.animation.new_scene("settle");
            for (k, col) in final_colors.iter().enumerate() {
                let sym = if k + 1 == final_colors.len() {
                    ch.input_symbol
                } else {
                    CIPHER[(id_n + k * 3) % CIPHER.len()]
                };
                settle.add_frame(
                    sym,
                    2,
                    Some(ColorPair {
                        fg: Some(*col),
                        bg: None,
                    }),
                );
            }
            ch.animation.activate_scene("scramble");
            ch.is_visible = true;
        }

        let n = ids.len();
        let scramble_frames = 18 + (n / 4).min(40);
        let reveal_stagger = 2usize;

        let mut out = Vec::new();
        for f in 0..scramble_frames {
            term.step_all();
            out.push(render_ansi(&term));
            let _ = f;
        }

        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                ch.animation.activate_scene("settle");
            }
            for _ in 0..reveal_stagger {
                term.step_all();
                out.push(render_ansi(&term));
            }
            let _ = i;
        }

        for _ in 0..12 {
            term.step_all();
            out.push(render_ansi(&term));
        }

        if out.is_empty() {
            out.push(render_ansi(&term));
        }
        out
    }
}

fn sgr(c: Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<(char, Option<Color>)>> = vec![vec![(' ', None); w]; h];
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
        if x >= w || y >= h {
            continue;
        }
        let vis = &ch.animation.current_character_visual;
        let fg = vis.colors.and_then(|p| p.fg).or_else(|| ch.colors.and_then(|p| p.fg));
        grid[y][x] = (vis.symbol, fg);
    }
    let mut s = String::new();
    for row in grid {
        for (sym, fg) in row {
            if let Some(c) = fg {
                s.push_str(&sgr(c));
                s.push(sym);
                s.push_str("\x1b[0m");
            } else {
                s.push(sym);
            }
        }
        s.push('\n');
    }
    s
}

#[allow(dead_code)]
fn _keep_character_ty(_: &EffectCharacter) {}
