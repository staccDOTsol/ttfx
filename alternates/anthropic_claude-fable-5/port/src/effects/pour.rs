//! Pour effect: characters pour down from the top of the canvas into place,
//! row by row in an alternating zig-zag order, shifting from a starting color
//! to a final vertical gradient color as they fall.
//!
//! Port of terminaltexteffects/effects/effect_pour.py (pour_direction=down).

use std::collections::{HashMap, VecDeque};

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// The `pour` effect with upstream default configuration.
pub struct Pour {
    /// Characters released per pour tick.
    pour_speed: usize,
    /// Speed of the falling characters (cells per tick).
    movement_speed: f64,
    /// Frames to wait between releasing characters.
    gap: u32,
    /// Color of characters when they first appear.
    starting_color: Color,
    /// Stops for the final (vertical) gradient across the canvas.
    final_gradient_stops: Vec<Color>,
    /// Interpolation steps between final gradient stops.
    final_gradient_steps: usize,
    /// Ticks each pour-gradient step is displayed.
    final_gradient_frames: u32,
}

impl Pour {
    pub fn new() -> Self {
        Self {
            pour_speed: 1,
            movement_speed: 0.2,
            gap: 1,
            starting_color: Color::from_hex("ffffff").expect("valid hex"),
            final_gradient_stops: vec![
                Color::from_hex("8A008A").expect("valid hex"),
                Color::from_hex("00D1FF").expect("valid hex"),
                Color::from_hex("FFFFFF").expect("valid hex"),
            ],
            final_gradient_steps: 12,
            final_gradient_frames: 10,
        }
    }
}

impl Default for Pour {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Pour {
    fn name(&self) -> &str {
        "pour"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;
        let top = height;

        // Final gradient mapped vertically across the canvas rows.
        let final_gradient = Gradient::new(&self.final_gradient_stops, self.final_gradient_steps);
        let mut final_color_map: HashMap<u32, Color> = HashMap::new();
        for character in terminal.get_characters() {
            let fraction = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                1.0
            };
            let color = final_gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(self.starting_color);
            final_color_map.insert(character.character_id, color);
        }

        // Group characters by row, top to bottom; columns ascending within a row.
        let mut placements: Vec<(u32, i32, i32)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.character_id, c.input_coord.row, c.input_coord.column))
            .collect();
        placements.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));

        let mut groups: Vec<Vec<u32>> = Vec::new();
        let mut current_row: Option<i32> = None;
        for (id, row, _col) in placements {
            if current_row != Some(row) {
                groups.push(Vec::new());
                current_row = Some(row);
            }
            groups.last_mut().expect("group exists").push(id);
        }
        // Alternate pour direction per row (odd-indexed rows pour right-to-left).
        let mut pending_groups: VecDeque<Vec<u32>> = VecDeque::new();
        for (i, mut group) in groups.into_iter().enumerate() {
            if i % 2 != 0 {
                group.reverse();
            }
            pending_groups.push_back(group);
        }

        // Per-character setup: start at the top, path to home, pour gradient scene.
        for character in terminal.get_characters_mut() {
            character.motion.current_coord = Coord::new(character.input_coord.column, top);

            let path = character
                .motion
                .new_path("input_coord", self.movement_speed, Some(easing::in_quad));
            path.new_waypoint("input_coord", character.input_coord);

            let final_color = final_color_map
                .get(&character.character_id)
                .copied()
                .unwrap_or(self.starting_color);
            let pour_gradient = Gradient::new(&[self.starting_color, final_color], 10);
            let scene = character.animation.new_scene("pour", false);
            for color in &pour_gradient.spectrum {
                scene.add_frame(
                    character.input_symbol,
                    self.final_gradient_frames,
                    Some(ColorPair::fg_only(*color)),
                );
            }
        }

        // Frame loop: release characters with the configured gap, then tick.
        let mut frames: Vec<String> = Vec::new();
        let mut current_group: Vec<u32> = Vec::new();
        let mut gap_remaining: u32 = 0;
        let max_frames = 20_000usize;

        loop {
            let has_pending = !pending_groups.is_empty() || !current_group.is_empty();
            if (!has_pending && !terminal.is_active()) || frames.len() >= max_frames {
                break;
            }

            if current_group.is_empty() {
                if let Some(group) = pending_groups.pop_front() {
                    current_group = group;
                }
            }

            if !current_group.is_empty() {
                if gap_remaining == 0 {
                    for _ in 0..self.pour_speed.max(1) {
                        if current_group.is_empty() {
                            break;
                        }
                        let id = current_group.remove(0);
                        terminal.set_character_visibility(id, true);
                        if let Some(character) = terminal
                            .get_characters_mut()
                            .iter_mut()
                            .find(|c| c.character_id == id)
                        {
                            character.animation.activate_scene("pour");
                            character.motion.activate_path("input_coord");
                        }
                    }
                    gap_remaining = self.gap;
                } else {
                    gap_remaining -= 1;
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());
        }

        frames
    }
}
