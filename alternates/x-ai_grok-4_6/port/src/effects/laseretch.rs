use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Laseretch;

impl Laseretch {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Laseretch {
    fn name(&self) -> &str {
        "laseretch"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let etch_stops = vec![
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
            Color::from_hex("ffd700").unwrap_or(Color::rgb(255, 215, 0)),
            Color::from_hex("ff4500").unwrap_or(Color::rgb(255, 69, 0)),
            Color::from_hex("8b0000").unwrap_or(Color::rgb(139, 0, 0)),
        ];
        let laser_stops = vec![
            Color::from_hex("00ffff").unwrap_or(Color::rgb(0, 255, 255)),
            Color::from_hex("0088ff").unwrap_or(Color::rgb(0, 136, 255)),
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let spark_stops = vec![
            Color::from_hex("ffff00").unwrap_or(Color::rgb(255, 255, 0)),
            Color::from_hex("ff8800").unwrap_or(Color::rgb(255, 136, 0)),
            Color::from_hex("ff0000").unwrap_or(Color::rgb(255, 0, 0)),
        ];
        let etch_colors = Gradient::new(etch_stops, 8).colors();
        let laser_colors = Gradient::new(laser_stops, 6).colors();
        let spark_colors = Gradient::new(spark_stops, 5).colors();

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                let scn = ch.animation.new_scene("etch");
                for col in &etch_colors {
                    scn.add_frame(
                        ch.input_symbol,
                        2,
                        Some(ColorPair {
                            fg: Some(*col),
                            bg: None,
                        }),
                    );
                }
                let last = *etch_colors.last().unwrap_or(&Color::rgb(255, 215, 0));
                scn.add_frame(
                    ch.input_symbol,
                    8,
                    Some(ColorPair {
                        fg: Some(last),
                        bg: None,
                    }),
                );
            }
        }

        // diagonal laser beam overlay
        let mut beam_ids: Vec<CharacterId> = Vec::new();
        {
            let mut row = 0i32;
            let mut col = 0i32;
            while row < height as i32 && col < width as i32 {
                let id = term.add_character('/', Coord::new(col, row));
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == id) {
                    ch.layer_placeholder();
                    let scn = ch.animation.new_scene("laser");
                    scn.is_looping = true;
                    for colr in &laser_colors {
                        scn.add_frame(
                            '*',
                            1,
                            Some(ColorPair {
                                fg: Some(*colr),
                                bg: None,
                            }),
                        );
                    }
                    ch.animation.activate_scene("laser");
                }
                term.set_character_visibility(id, true);
                beam_ids.push(id);
                row += 1;
                col += 1;
            }
        }

        // sparks pool
        let mut spark_ids: Vec<CharacterId> = Vec::new();
        for i in 0..12 {
            let id = term.add_character('.', Coord::new((i * 3) as i32 % width as i32, 0));
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == id) {
                let scn = ch.animation.new_scene("spark");
                for colr in &spark_colors {
                    scn.add_frame(
                        '*',
                        1,
                        Some(ColorPair {
                            fg: Some(*colr),
                            bg: None,
                        }),
                    );
                }
                ch.animation.activate_scene("spark");
            }
            spark_ids.push(id);
        }

        let mut frames: Vec<String> = Vec::new();
        let n = ids.len();
        for i in 0..n {
            term.set_character_visibility(ids[i], true);
            if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == ids[i]) {
                ch.animation.activate_scene("etch");
            }
            // move sparks near last etched char
            if let Some(etch) = term.get_characters().iter().find(|c| c.id == ids[i]) {
                let base = etch.input_coord;
                for (si, sid) in spark_ids.iter().enumerate() {
                    if let Some(sp) = term.get_characters_mut().iter_mut().find(|c| c.id == *sid) {
                        sp.motion.current_coord = Coord::new(
                            base.column + ((si as i32) % 5) - 2,
                            base.row + ((si as i32) / 5) - 1,
                        );
                        sp.current_coord = sp.motion.current_coord;
                    }
                    term.set_character_visibility(*sid, true);
                }
            }
            term.step_all();
            frames.push(render_ansi(&term));
        }

        // hold etched text while laser fades
        for _ in 0..12 {
            term.step_all();
            frames.push(render_ansi(&term));
        }
        for bid in beam_ids {
            term.set_character_visibility(bid, false);
        }
        for sid in spark_ids {
            term.set_character_visibility(sid, false);
        }
        for _ in 0..8 {
            term.step_all();
            frames.push(render_ansi(&term));
        }
        if frames.is_empty() {
            frames.push(render_ansi(&term));
        }
        frames
    }
}

trait LayerPh {
    fn layer_placeholder(&mut self) {}
}

impl LayerPh for crate::engine::character::EffectCharacter {}

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
        let ux = x as usize;
        let uy = y as usize;
        if ux >= w || uy >= h {
            continue;
        }
        let sym = ch.animation.current_character_visual.symbol;
        let col = ch
            .animation
            .current_character_visual
            .colors
            .and_then(|p| p.fg)
            .or_else(|| ch.colors.and_then(|p| p.fg));
        grid[uy][ux] = (sym, col);
    }
    let mut out = String::new();
    for row in grid {
        for (sym, col) in row {
            if let Some(c) = col {
                out.push_str(&sgr(c));
                out.push(sym);
                out.push_str("\x1b[0m");
            } else if sym != ' ' && sym != '\0' {
                out.push_str("\x1b[38;2;255;215;0m");
                out.push(sym);
                out.push_str("\x1b[0m");
            } else {
                out.push(' ');
            }
        }
        out.push('\n');
    }
    out
}
