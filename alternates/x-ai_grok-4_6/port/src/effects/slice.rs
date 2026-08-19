use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Slice;

impl Slice {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Slice {
    fn name(&self) -> &str {
        "slice"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height);

        let stops = vec![
            Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
            Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
            Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
        ];
        let gradient = Gradient::new(stops, 8);
        let palette = gradient.colors();

        let mut rows: Vec<Vec<CharacterId>> = vec![Vec::new(); height];
        for ch in term.get_characters() {
            let r = ch.input_coord.row;
            if r >= 0 && (r as usize) < height {
                rows[r as usize].push(ch.id);
            }
        }
        for row in &mut rows {
            row.sort_by_key(|id| {
                term.get_characters()
                    .iter()
                    .find(|c| c.id == *id)
                    .map(|c| c.input_coord.column)
                    .unwrap_or(0)
            });
        }

        let center_col = (width as i32) / 2;
        let mut path_ids: Vec<(CharacterId, String)> = Vec::new();

        for (row_index, row) in rows.iter().enumerate() {
            let left: Vec<CharacterId> = row
                .iter()
                .copied()
                .filter(|id| {
                    term.get_characters()
                        .iter()
                        .find(|c| c.id == *id)
                        .map(|c| c.input_coord.column <= center_col)
                        .unwrap_or(false)
                })
                .collect();
            for id in &left {
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    let dest = ch.input_coord;
                    ch.motion.current_coord = Coord::new(dest.column, height as i32 + 1);
                    ch.current_coord = ch.motion.current_coord;
                    let pid = format!("in-{}", id.0);
                    {
                        let p = ch.motion.new_path(pid.clone());
                        p.speed = 0.5;
                        p.easing = Easing::OutQuad;
                        p.new_waypoint("dest", dest);
                    }
                    ch.motion.activate_path(&pid);
                    path_ids.push((*id, pid));
                    let color = palette.get(row_index % palette.len()).copied();
                    let scn = ch.animation.new_scene("slice");
                    scn.add_frame(
                        ch.input_symbol,
                        1,
                        Some(ColorPair {
                            fg: color,
                            bg: None,
                        }),
                    );
                    ch.animation.activate_scene("slice");
                }
                term.set_character_visibility(*id, true);
            }

            let opp = height.saturating_sub(row_index + 1);
            let right: Vec<CharacterId> = if opp < rows.len() {
                rows[opp]
                    .iter()
                    .copied()
                    .filter(|id| {
                        term.get_characters()
                            .iter()
                            .find(|c| c.id == *id)
                            .map(|c| c.input_coord.column > center_col)
                            .unwrap_or(false)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            for id in &right {
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    let dest = ch.input_coord;
                    ch.motion.current_coord = Coord::new(dest.column, -1);
                    ch.current_coord = ch.motion.current_coord;
                    let pid = format!("in-{}", id.0);
                    {
                        let p = ch.motion.new_path(pid.clone());
                        p.speed = 0.5;
                        p.easing = Easing::OutQuad;
                        p.new_waypoint("dest", dest);
                    }
                    ch.motion.activate_path(&pid);
                    path_ids.push((*id, pid));
                    let color = palette.get((row_index + 3) % palette.len()).copied();
                    let scn = ch.animation.new_scene("slice");
                    scn.add_frame(
                        ch.input_symbol,
                        1,
                        Some(ColorPair {
                            fg: color,
                            bg: None,
                        }),
                    );
                    ch.animation.activate_scene("slice");
                }
                term.set_character_visibility(*id, true);
            }
        }

        let mut out = Vec::new();
        for _ in 0..80 {
            term.step_all();
            let mut frame = String::new();
            for y in 0..height {
                for x in 0..width {
                    let coord = Coord::new(x as i32, y as i32);
                    let mut painted = false;
                    for ch in term.get_characters() {
                        if ch.is_visible && ch.current_coord == coord {
                            let vis = &ch.animation.current_character_visual;
                            if let Some(pair) = vis.colors.or(ch.colors) {
                                if let Some(fg) = pair.fg {
                                    frame.push_str(&format!(
                                        "\x1b[38;2;{};{};{}m{}\x1b[0m",
                                        fg.r, fg.g, fg.b, vis.symbol
                                    ));
                                    painted = true;
                                    break;
                                }
                            }
                            frame.push(vis.symbol);
                            painted = true;
                            break;
                        }
                    }
                    if !painted {
                        frame.push(' ');
                    }
                }
                frame.push('\n');
            }
            out.push(frame);
        }
        out
    }
}
