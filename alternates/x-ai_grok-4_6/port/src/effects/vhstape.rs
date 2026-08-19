use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const GLITCH_SYMBOLS: &[char] = &['█', '▓', '▒', '░', '▄', '▀', '▌', '▐'];

pub struct Vhstape;

impl Vhstape {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Vhstape {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Vhstape {
    fn name(&self) -> &str {
        "vhstape"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = if input.is_empty() {
            vec!["vhs"]
        } else {
            input.lines().collect()
        };
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);

        let mut term = Terminal::from_input(input, width, height);
        if term.get_characters().is_empty() {
            term.add_character('V', Coord::new(0, 0));
        }

        let tape_stops = vec![
            Color::from_hex("8A2BE2").unwrap_or(Color::rgb(138, 43, 226)),
            Color::from_hex("00FFFF").unwrap_or(Color::rgb(0, 255, 255)),
            Color::from_hex("FF00FF").unwrap_or(Color::rgb(255, 0, 255)),
            Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let glitch_stops = vec![
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
            Color::from_hex("ff0000").unwrap_or(Color::rgb(255, 0, 0)),
            Color::from_hex("00ff00").unwrap_or(Color::rgb(0, 255, 0)),
            Color::from_hex("0000ff").unwrap_or(Color::rgb(0, 0, 255)),
            Color::from_hex("ffffff").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let tape_grad = Gradient::new(tape_stops, 12).colors();
        let glitch_grad = Gradient::new(glitch_stops, 8).colors();
        let snow_color = Color::from_hex("c0c0c0").unwrap_or(Color::rgb(192, 192, 192));

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        // Build per-character scenes: base + glitch (mirrors Python vhstape scenes)
        {
            let chars = term.get_characters_mut();
            for (i, ch) in chars.iter_mut().enumerate() {
                let base_c = tape_grad[i % tape_grad.len()];
                let pair = ColorPair {
                    fg: Some(base_c),
                    bg: None,
                };
                ch.colors = Some(pair);
                {
                    let scn = ch.animation.new_scene("base");
                    scn.add_frame(ch.input_symbol, 4, Some(pair));
                }
                {
                    let scn = ch.animation.new_scene("glitch");
                    for (gi, &sym) in GLITCH_SYMBOLS.iter().enumerate() {
                        let gc = glitch_grad[gi % glitch_grad.len()];
                        scn.add_frame(
                            sym,
                            1,
                            Some(ColorPair {
                                fg: Some(gc),
                                bg: None,
                            }),
                        );
                    }
                    scn.add_frame(ch.input_symbol, 2, Some(pair));
                }
                {
                    let scn = ch.animation.new_scene("snow");
                    scn.add_frame(
                        '.',
                        1,
                        Some(ColorPair {
                            fg: Some(snow_color),
                            bg: None,
                        }),
                    );
                    scn.add_frame(ch.input_symbol, 1, Some(pair));
                }
                ch.animation.activate_scene("base");

                let wave = ch.motion.new_path("glitch_wave");
                wave.speed = 2.0;
                wave.new_waypoint(
                    "mid",
                    Coord::new(ch.input_coord.column + 8, ch.input_coord.row),
                );
                wave.new_waypoint("end", ch.input_coord);

                let endp = ch.motion.new_path("glitch_wave_end");
                endp.speed = 2.0;
                endp.new_waypoint(
                    "glitch_wave_end",
                    Coord::new(ch.input_coord.column + 14, ch.input_coord.row),
                );
            }
        }

        let n = ids.len().max(1);
        let total_frames = 48 + n.min(40);

        let mut out = Vec::with_capacity(total_frames);
        for f in 0..total_frames {
            let phase = f % 16;
            {
                let chars = term.get_characters_mut();
                for (i, ch) in chars.iter_mut().enumerate() {
                    let snow_row = ((f / 2) + i) % (height + 3);
                    if ch.input_coord.row == snow_row as i32 {
                        ch.animation.activate_scene("snow");
                    } else if phase == (i % 16) || phase == ((i + 7) % 16) {
                        ch.animation.activate_scene("glitch");
                        if phase < 4 {
                            ch.motion.activate_path("glitch_wave");
                        } else if phase > 12 {
                            ch.motion.activate_path("glitch_wave_end");
                        }
                    } else {
                        ch.animation.activate_scene("base");
                    }
                }
            }
            term.step_all();

            // Manual SGR render (canvas has no color escapes)
            let mut frame = String::new();
            let mut grid: Vec<Vec<(char, Option<Color>)>> =
                vec![vec![(' ', None); width]; height];
            for ch in term.get_characters() {
                if !ch.is_visible {
                    continue;
                }
                let col = ch.current_coord.column;
                let row = ch.current_coord.row;
                if col >= 0 && row >= 0 && (col as usize) < width && (row as usize) < height {
                    let vis = &ch.animation.current_character_visual;
                    let color = vis
                        .colors
                        .and_then(|p| p.fg)
                        .or_else(|| ch.colors.and_then(|p| p.fg));
                    grid[row as usize][col as usize] = (vis.symbol, color);
                }
            }
            for row in grid {
                for (sym, col) in row {
                    if let Some(c) = col {
                        frame.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, sym));
                    } else {
                        frame.push(sym);
                    }
                }
                frame.push('\n');
            }
            out.push(frame);
        }
        out
    }
}
