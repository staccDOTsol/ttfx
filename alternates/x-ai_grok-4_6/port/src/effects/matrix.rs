use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const KATAKANA: &[char] = &[
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ',
    'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ',
    'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

pub struct Matrix;

impl Matrix {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Matrix {
    fn name(&self) -> &str {
        "matrix"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let width = input
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(40)
            .max(1);
        let height = input.lines().count().max(1);
        let mut term = Terminal::from_input(input, width, height.max(8));

        let rain_stops = vec![
            Color::from_hex("00ff00").unwrap_or(Color::rgb(0, 255, 0)),
            Color::from_hex("006600").unwrap_or(Color::rgb(0, 102, 0)),
            Color::from_hex("002200").unwrap_or(Color::rgb(0, 34, 0)),
        ];
        let rain_grad = Gradient::new(rain_stops, 12);
        let rain_colors = rain_grad.colors();
        let head_color = Color::from_hex("ccffcc").unwrap_or(Color::rgb(204, 255, 204));
        let resolve_color = Color::from_hex("00aa00").unwrap_or(Color::rgb(0, 170, 0));

        let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut next_u32 = |s: &mut u64| {
            *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            (*s >> 32) as u32
        };

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        // rain overlay characters per column
        let mut rain_ids: Vec<CharacterId> = Vec::new();
        for col in 0..width as i32 {
            let start_row = -((next_u32(&mut rng) % (height as u32 * 2 + 4)) as i32);
            let trail = 4 + (next_u32(&mut rng) % 10) as i32;
            for t in 0..trail {
                let row = start_row - t;
                let sym = KATAKANA[(next_u32(&mut rng) as usize) % KATAKANA.len()];
                let id = term.add_character(sym, Coord::new(col, row));
                rain_ids.push(id);
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == id) {
                    let color = if t == 0 {
                        head_color
                    } else {
                        rain_colors[(t as usize).min(rain_colors.len().saturating_sub(1))]
                    };
                    let scene = ch.animation.new_scene("rain");
                    scene.is_looping = true;
                    for k in 0..6 {
                        let s = KATAKANA[(next_u32(&mut rng) as usize + k) % KATAKANA.len()];
                        scene.add_frame(
                            s,
                            2 + (k as u32 % 3),
                            Some(ColorPair {
                                fg: Some(color),
                                bg: None,
                            }),
                        );
                    }
                    ch.animation.activate_scene("rain");
                    ch.animation.set_appearance(
                        sym,
                        Some(color),
                    );
                }
                term.set_character_visibility(id, true);
            }
        }

        // resolve scenes on input glyphs
        for ch in term.get_characters_mut() {
            if rain_ids.iter().any(|r| *r == ch.id) {
                continue;
            }
            let scene = ch.animation.new_scene("resolve");
            for i in 0..8 {
                let s = KATAKANA[(next_u32(&mut rng) as usize + i) % KATAKANA.len()];
                let c = rain_colors[i % rain_colors.len()];
                scene.add_frame(
                    s,
                    2,
                    Some(ColorPair {
                        fg: Some(c),
                        bg: None,
                    }),
                );
            }
            scene.add_frame(
                ch.input_symbol,
                8,
                Some(ColorPair {
                    fg: Some(resolve_color),
                    bg: None,
                }),
            );
            ch.animation.activate_scene("resolve");
        }

        let mut frames = Vec::new();
        let total = (height as i32 + 28).max(40) as usize;
        for tick in 0..total {
            // advance rain downward
            for id in &rain_ids {
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    ch.motion.current_coord.row += 1;
                    if ch.motion.current_coord.row > height as i32 + 2 {
                        ch.motion.current_coord.row = -((next_u32(&mut rng) % 8) as i32);
                    }
                    ch.current_coord = ch.motion.current_coord;
                    let col_idx = (tick + ch.id.0 as usize) % rain_colors.len();
                    let fg = if (ch.current_coord.row + tick as i32) % 7 == 0 {
                        head_color
                    } else {
                        rain_colors[col_idx]
                    };
                    ch.animation.set_appearance(
                        KATAKANA[(next_u32(&mut rng) as usize) % KATAKANA.len()],
                        Some(fg),
                    );
                }
            }

            term.step_all();

            // paint with SGR
            let mut out = String::new();
            let w = term.canvas.width;
            let h = term.canvas.height;
            let mut grid: Vec<Vec<(char, Option<Color>)>> =
                vec![vec![(' ', None); w]; h];
            for ch in term.get_characters() {
                if !ch.is_visible {
                    continue;
                }
                let x = ch.current_coord.column;
                let y = ch.current_coord.row;
                if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                    let sym = ch.animation.current_character_visual.symbol;
                    let fg = ch
                        .animation
                        .current_character_visual
                        .colors
                        .and_then(|p| p.fg)
                        .or_else(|| ch.colors.and_then(|p| p.fg));
                    grid[y as usize][x as usize] = (sym, fg);
                }
            }
            for row in grid {
                for (sym, fg) in row {
                    if let Some(c) = fg {
                        out.push_str(&format!(
                            "\x1b[38;2;{};{};{}m{}\x1b[0m",
                            c.r, c.g, c.b, sym
                        ));
                    } else if sym != ' ' {
                        out.push_str("\x1b[32m");
                        out.push(sym);
                        out.push_str("\x1b[0m");
                    } else {
                        out.push(' ');
                    }
                }
                out.push('\n');
            }
            frames.push(out);
        }
        frames
    }
}
