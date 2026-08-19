use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Smoke;

impl Smoke {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Smoke {
    fn name(&self) -> &str {
        "smoke"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height + 8);

        let smoke_stops = vec![
            Color::from_hex("333333").unwrap_or(Color::rgb(0x33, 0x33, 0x33)),
            Color::from_hex("666666").unwrap_or(Color::rgb(0x66, 0x66, 0x66)),
            Color::from_hex("999999").unwrap_or(Color::rgb(0x99, 0x99, 0x99)),
            Color::from_hex("CCCCCC").unwrap_or(Color::rgb(0xcc, 0xcc, 0xcc)),
            Color::from_hex("FFFFFF").unwrap_or(Color::rgb(0xff, 0xff, 0xff)),
        ];
        let gradient = Gradient::new(smoke_stops, 12);
        let palette = gradient.colors();

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for (i, id) in ids.iter().enumerate() {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let color = palette[i % palette.len()];
                let rise = Coord::new(ch.input_coord.column, (ch.input_coord.row - 6).max(0));
                {
                    let path = ch.motion.new_path("rise");
                    path.speed = 0.35 + ((i % 5) as f64) * 0.08;
                    path.easing = Easing::OutSine;
                    path.new_waypoint("top", rise);
                }
                {
                    let scn = ch.animation.new_scene("smoke");
                    scn.is_looping = true;
                    for (fi, col) in palette.iter().enumerate() {
                        let sym = match fi % 4 {
                            0 => ch.input_symbol,
                            1 => '░',
                            2 => '▒',
                            _ => '▓',
                        };
                        scn.add_frame(
                            sym,
                            2,
                            Some(ColorPair {
                                fg: Some(*col),
                                bg: None,
                            }),
                        );
                    }
                }
                ch.animation.activate_scene("smoke");
                ch.motion.activate_path("rise");
                ch.animation.set_appearance(ch.input_symbol, Some(color));
            }
            term.set_character_visibility(*id, true);
        }

        let mut out = Vec::new();
        for tick in 0..48 {
            term.step_all();
            let mut frame = String::new();
            let chars: Vec<(Coord, char, Option<ColorPair>)> = term
                .get_characters()
                .iter()
                .filter(|c| c.is_visible)
                .map(|c| {
                    (
                        c.current_coord,
                        c.animation.current_character_visual.symbol,
                        c.animation
                            .current_character_visual
                            .colors
                            .or(c.colors)
                            .or_else(|| {
                                Some(ColorPair {
                                    fg: Some(palette[tick % palette.len()]),
                                    bg: None,
                                })
                            }),
                    )
                })
                .collect();
            let mut grid = vec![vec![' '; width]; height + 8];
            let mut cols = vec![vec![None; width]; height + 8];
            for (coord, sym, pair) in chars {
                let x = coord.column;
                let y = coord.row;
                if x >= 0 && y >= 0 {
                    let ux = x as usize;
                    let uy = y as usize;
                    if ux < width && uy < height + 8 {
                        grid[uy][ux] = sym;
                        cols[uy][ux] = pair.and_then(|p| p.fg);
                    }
                }
            }
            for y in 0..height + 8 {
                for x in 0..width {
                    if let Some(c) = cols[y][x] {
                        frame.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, grid[y][x]));
                    } else {
                        frame.push(grid[y][x]);
                    }
                }
                frame.push('\n');
            }
            out.push(frame);
        }
        out
    }
}
