//! OrbittingVolley: four launchers orbit the canvas perimeter, firing the
//! input characters toward their home coordinates in volleys.
//! Port of terminaltexteffects/effects/effect_orbittingvolley.py.

use std::cmp::Ordering;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_length_of_line, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

const LAUNCHER_SYMBOL: char = '█';
const LAUNCHER_MOVEMENT_SPEED: f64 = 0.5;
const CHARACTER_MOVEMENT_SPEED: f64 = 1.0;
const VOLLEY_SIZE: f64 = 0.03;
const LAUNCH_DELAY: u32 = 15;
const MAX_FRAMES: usize = 2000;

pub struct Orbittingvolley;

impl Orbittingvolley {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Orbittingvolley {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Orbittingvolley {
    fn name(&self) -> &str {
        "orbittingvolley"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.config.width;
        let height = terminal.config.height;

        // Final gradient (Python default stops: FFA15C -> 44D492).
        let stops = [
            Color::from_hex("FFA15C").expect("valid hex"),
            Color::from_hex("44D492").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&stops, 12);
        let fallback_color = stops[0];

        // Perimeter corners, clockwise from the top-left.
        let corners = [
            Coord::new(1, height),
            Coord::new(width, height),
            Coord::new(width, 1),
            Coord::new(1, 1),
        ];

        // Snapshot the text characters (ids equal arena indices by construction).
        let text_chars: Vec<(u32, Coord)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.character_id, c.input_coord))
            .collect();

        if text_chars.is_empty() {
            return vec![terminal.render_frame()];
        }

        // Style each character from the final gradient (diagonal direction)
        // and give it its "input_path" back home.
        for &(id, coord) in &text_chars {
            let col_t = if width > 1 {
                (coord.column - 1) as f64 / (width - 1) as f64
            } else {
                0.0
            };
            let row_t = if height > 1 {
                (coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let t = (col_t + row_t) / 2.0;
            let color = final_gradient
                .get_color_at_fraction(t)
                .unwrap_or(fallback_color);

            let character = &mut terminal.get_characters_mut()[id as usize];
            let symbol = character.input_symbol;
            character
                .animation
                .set_appearance(symbol, Some(ColorPair::fg_only(color)));
            let input_path = character.motion.new_path(
                "input_path",
                CHARACTER_MOVEMENT_SPEED,
                Some(easing::out_sine),
            );
            input_path.new_waypoint("input_coord", coord);
        }

        // Build the four launchers, one per corner, each with a looping
        // gradient "spin" scene and a perimeter path visiting the corners
        // in clockwise order starting from its own corner.
        let mut launcher_ids: Vec<u32> = Vec::with_capacity(4);
        for (corner_index, corner) in corners.iter().enumerate() {
            let id = terminal.add_character(LAUNCHER_SYMBOL, *corner);
            launcher_ids.push(id);
            let launcher = &mut terminal.get_characters_mut()[id as usize];
            launcher.is_visible = true;

            let spin_scn = launcher.animation.new_scene("spin", true);
            for color in &final_gradient.spectrum {
                spin_scn.add_frame(LAUNCHER_SYMBOL, 3, Some(ColorPair::fg_only(*color)));
            }
            launcher.animation.activate_scene("spin");

            let perimeter_path =
                launcher
                    .motion
                    .new_path("perimeter", LAUNCHER_MOVEMENT_SPEED, None);
            for step in 1..=4usize {
                let waypoint_coord = corners[(corner_index + step) % 4];
                perimeter_path.new_waypoint(&step.to_string(), waypoint_coord);
            }
            launcher.motion.activate_path("perimeter");
        }

        // Load each character into the magazine of the nearest launcher corner.
        let mut magazines: [Vec<u32>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for &(id, coord) in &text_chars {
            let nearest = corners
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    find_length_of_line(coord, **a)
                        .partial_cmp(&find_length_of_line(coord, **b))
                        .unwrap_or(Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            magazines[nearest].push(id);
        }

        let volley_count = ((text_chars.len() as f64 * VOLLEY_SIZE).floor() as usize).max(1);

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let mut tick_count: u32 = 0;
        loop {
            // Fire a volley from each launcher on the launch cadence.
            if tick_count % LAUNCH_DELAY == 0 {
                for (launcher_index, launcher_id) in launcher_ids.iter().enumerate() {
                    let launcher_coord = terminal.get_characters()[*launcher_id as usize]
                        .motion
                        .current_coord;
                    for _ in 0..volley_count {
                        if magazines[launcher_index].is_empty() {
                            break;
                        }
                        let char_id = magazines[launcher_index].remove(0);
                        let character = &mut terminal.get_characters_mut()[char_id as usize];
                        character.motion.current_coord = launcher_coord;
                        character.is_visible = true;
                        character.motion.activate_path("input_path");
                    }
                }
            }

            // Keep the launchers orbiting the perimeter forever.
            for launcher_id in &launcher_ids {
                let launcher = &mut terminal.get_characters_mut()[*launcher_id as usize];
                if launcher.motion.movement_is_complete() {
                    launcher.motion.activate_path("perimeter");
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());
            tick_count += 1;

            let magazines_empty = magazines.iter().all(Vec::is_empty);
            let characters_settled = text_chars.iter().all(|&(id, _)| {
                terminal.get_characters()[id as usize]
                    .motion
                    .movement_is_complete()
            });
            if (magazines_empty && characters_settled) || frames.len() >= MAX_FRAMES {
                break;
            }
        }

        // Retire the launchers and show the settled, gradient-colored text.
        for launcher_id in &launcher_ids {
            terminal.set_character_visibility(*launcher_id, false);
        }
        frames.push(terminal.render_frame());

        frames
    }
}
