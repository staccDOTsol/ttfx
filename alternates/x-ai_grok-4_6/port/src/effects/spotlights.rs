use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Spotlights;

impl Spotlights {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Spotlights {
    fn name(&self) -> &str {
        "spotlights"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height.max(8));
        if term.get_characters().is_empty() {
            for (row, line) in lines.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    term.add_character(ch, Coord::new(col as i32, row as i32));
                }
            }
        }

        let beam = Gradient::new(
            vec![
                Color::rgb(0x10, 0x10, 0x18),
                Color::rgb(0xff, 0xf4, 0xa3),
                Color::rgb(0xff, 0xff, 0xff),
            ],
            8,
        )
        .colors();
        let idle = Color::rgb(0x2a, 0x2a, 0x38);

        let ids: Vec<CharacterId> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        // virtual spotlight wanderers
        let mut spots: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
        let n_spots = 3usize;
        for i in 0..n_spots {
            let a = (i as f64) * std::f64::consts::TAU / n_spots as f64;
            spots.push((
                width as f64 * 0.5 + (width as f64 * 0.25) * a.cos(),
                height as f64 * 0.5 + (height as f64 * 0.25) * a.sin(),
                0.35 + i as f64 * 0.07,
                a,
                4.0 + i as f64,
            ));
        }

        let mut frames = Vec::new();
        let total = 90usize.max(ids.len() + 20);
        for tick in 0..total {
            for spot in &mut spots {
                spot.3 += 0.08 + spot.2 * 0.02;
                let tx = width as f64 * 0.5 + (width as f64 * 0.42) * spot.3.cos();
                let ty = height as f64 * 0.5 + (height as f64 * 0.38) * (spot.3 * 1.3).sin();
                spot.0 += (tx - spot.0) * 0.12;
                spot.1 += (ty - spot.1) * 0.12;
            }

            {
                let chars = term.get_characters_mut();
                for ch in chars.iter_mut() {
                    let mut lit = 0.0f64;
                    for spot in &spots {
                        let d = ((ch.input_coord.column as f64 - spot.0).powi(2)
                            + (ch.input_coord.row as f64 - spot.1).powi(2))
                        .sqrt();
                        let fall = (1.0 - (d / spot.4).min(1.0)).max(0.0);
                        lit = lit.max(fall);
                    }
                    let idx = ((lit * (beam.len().saturating_sub(1) as f64)).round() as usize)
                        .min(beam.len().saturating_sub(1));
                    let color = if lit < 0.08 { idle } else { beam[idx] };
                    let scn_id = format!("s{}", tick);
                    {
                        let scn = ch.animation.new_scene(scn_id.clone());
                        scn.add_frame(
                            ch.input_symbol,
                            1,
                            Some(ColorPair {
                                fg: Some(color),
                                bg: None,
                            }),
                        );
                    }
                    ch.animation.activate_scene(&scn_id);
                    if tick == 0 {
                        let p = ch.motion.new_path("hold");
                        p.speed = 0.2;
                        p.easing = Easing::InOutSine;
                        p.new_waypoint("h", ch.input_coord);
                        ch.motion.activate_path("hold");
                    }
                }
            }
            term.step_all();
            let raw = term.get_formatted_output_string();
            frames.push(colorize_frame(&raw, &term, &beam, idle, &spots));
        }
        frames
    }
}

fn colorize_frame(
    raw: &str,
    term: &Terminal,
    beam: &[Color],
    idle: Color,
    spots: &[(f64, f64, f64, f64, f64)],
) -> String {
    let mut out = String::new();
    for (y, line) in raw.lines().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == ' ' {
                out.push(' ');
                continue;
            }
            let mut lit = 0.0f64;
            for spot in spots {
                let d = ((x as f64 - spot.0).powi(2) + (y as f64 - spot.1).powi(2)).sqrt();
                let fall = (1.0 - (d / spot.4).min(1.0)).max(0.0);
                lit = lit.max(fall);
            }
            let idx = ((lit * (beam.len().saturating_sub(1) as f64)).round() as usize)
                .min(beam.len().saturating_sub(1));
            let c = if lit < 0.08 { idle } else { beam[idx] };
            // prefer engine visual if present
            let vis = term.get_characters().iter().find(|ec| {
                ec.current_coord.column == x as i32 && ec.current_coord.row == y as i32
            });
            let col = vis
                .and_then(|ec| {
                    ec.animation
                        .current_character_visual
                        .colors
                        .and_then(|p| p.fg)
                })
                .unwrap_or(c);
            out.push_str(&format!(
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                col.r, col.g, col.b, ch
            ));
        }
        out.push('\n');
    }
    out
}
