//! binarypath: characters are replaced by their binary representation, whose
//! bits travel from outside the canvas to the character's input coordinate.
//! When all bits arrive, the character collapses into view, and a final
//! diagonal wipe brightens everything to the final gradient color.
//!
//! Rust port of terminaltexteffects/effects/effect_binarypath.py.

use std::collections::{BTreeMap, HashSet, VecDeque};

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Small deterministic PRNG (LCG) so we avoid external crates.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    /// Uniform-ish value in `0..n` (n clamped to at least 1).
    fn gen_range(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n.max(1)
    }
}

/// Mirror of Python's `_BinaryRepresentation`: one source character plus the
/// binary characters that travel toward it.
struct BinaryRep {
    source_id: u32,
    bin_ids: Vec<u32>,
}

fn dimmed(c: Color, factor: f64) -> Color {
    let scale = |v: u8| -> u8 { ((v as f64) * factor).round().clamp(0.0, 255.0) as u8 };
    Color::new(scale(c.r), scale(c.g), scale(c.b))
}

pub struct Binarypath;

impl Binarypath {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Binarypath {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Binarypath {
    fn name(&self) -> &str {
        "binarypath"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let mut rng = Rng::new(0x5eed_b175 ^ input.len() as u64);

        // --- config defaults (from BinaryPathConfig) ---
        let final_stops = [
            Color::from_hex("00d500").expect("valid hex"),
            Color::from_hex("007500").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&final_stops, 12);
        let binary_colors: Vec<Color> = ["044E29", "157e38", "45bf55", "95ed87"]
            .iter()
            .map(|h| Color::from_hex(h).expect("valid hex"))
            .collect();
        let white = Color::new(255, 255, 255);
        let movement_speed = 1.0;
        let active_binary_groups = 0.05_f64;

        let width = terminal.canvas.width;
        let height = terminal.canvas.height;

        // Snapshot the source (input) characters before adding binary characters.
        let sources: Vec<(u32, char, Coord)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.character_id, c.input_symbol, c.input_coord))
            .collect();

        // --- build: scenes for source characters, binary representations ---
        let mut pending: Vec<BinaryRep> = Vec::new();

        for &(id, symbol, coord) in &sources {
            // Final color from a vertical gradient mapping across the canvas.
            let t = if height > 1 {
                1.0 - (coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient
                .get_color_at_fraction(t)
                .unwrap_or(final_stops[0]);
            let dim_color = dimmed(final_color, 0.5);

            {
                let character = &mut terminal.get_characters_mut()[id as usize];
                // collapse_scn: white -> dim final color, as the bits collapse in.
                let collapse_scn = character.animation.new_scene("collapse_scn", false);
                for color in Gradient::new(&[white, dim_color], 10).spectrum {
                    collapse_scn.add_frame(symbol, 7, Some(ColorPair::fg_only(color)));
                }
                // brighten_scn: dim -> final color for the closing wipe.
                let brighten_scn = character.animation.new_scene("brighten_scn", false);
                for color in Gradient::new(&[dim_color, final_color], 10).spectrum {
                    brighten_scn.add_frame(symbol, 2, Some(ColorPair::fg_only(color)));
                }
            }

            // Binary representation: one traveling character per bit of the symbol.
            let bits = format!("{:b}", symbol as u32);
            let mut bin_ids: Vec<u32> = Vec::with_capacity(bits.len());
            for bit in bits.chars() {
                let start = match rng.gen_range(4) {
                    0 => Coord::new(0, 1 + rng.gen_range(height as usize) as i32),
                    1 => Coord::new(width + 1, 1 + rng.gen_range(height as usize) as i32),
                    2 => Coord::new(1 + rng.gen_range(width as usize) as i32, 0),
                    _ => Coord::new(1 + rng.gen_range(width as usize) as i32, height + 1),
                };
                let bid = terminal.add_character(bit, coord);
                let bin_char = &mut terminal.get_characters_mut()[bid as usize];
                bin_char.motion.current_coord = start;
                let path = bin_char.motion.new_path("input_coord", movement_speed, None);
                path.new_waypoint("target", coord);
                let color = binary_colors[rng.gen_range(binary_colors.len())];
                bin_char
                    .animation
                    .set_appearance(bit, Some(ColorPair::fg_only(color)));
                bin_ids.push(bid);
            }
            pending.push(BinaryRep {
                source_id: id,
                bin_ids,
            });
        }

        let max_active_binary_groups =
            ((pending.len() as f64 * active_binary_groups) as usize).max(1);

        // Diagonal groups for the final wipe (bottom-left to top-right).
        let mut diagonal_groups: BTreeMap<i32, Vec<u32>> = BTreeMap::new();
        for &(id, _, coord) in &sources {
            diagonal_groups
                .entry(coord.column + coord.row)
                .or_default()
                .push(id);
        }
        let mut wipe_groups: VecDeque<Vec<u32>> = diagonal_groups.into_values().collect();

        // --- frame loop ---
        let mut active_reps: Vec<BinaryRep> = Vec::new();
        let mut active_ids: HashSet<u32> = HashSet::new();
        let mut travel_phase = true;

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let max_frames = 5000usize;
        let mut frame_count = 0usize;

        loop {
            if travel_phase {
                // Activate binary representations up to the concurrency cap.
                while active_reps.len() < max_active_binary_groups && !pending.is_empty() {
                    let idx = rng.gen_range(pending.len());
                    let rep = pending.swap_remove(idx);
                    for &bid in &rep.bin_ids {
                        terminal.set_character_visibility(bid, true);
                        terminal.get_characters_mut()[bid as usize]
                            .motion
                            .activate_path("input_coord");
                        active_ids.insert(bid);
                    }
                    active_reps.push(rep);
                }

                // Collapse any representation whose bits have all arrived.
                let mut still_active: Vec<BinaryRep> = Vec::new();
                for rep in active_reps.drain(..) {
                    let travel_complete = rep.bin_ids.iter().all(|&bid| {
                        terminal.get_characters()[bid as usize]
                            .motion
                            .movement_is_complete()
                    });
                    if travel_complete {
                        for &bid in &rep.bin_ids {
                            terminal.set_character_visibility(bid, false);
                            active_ids.remove(&bid);
                        }
                        terminal.set_character_visibility(rep.source_id, true);
                        terminal.get_characters_mut()[rep.source_id as usize]
                            .animation
                            .activate_scene("collapse_scn");
                        active_ids.insert(rep.source_id);
                    } else {
                        still_active.push(rep);
                    }
                }
                active_reps = still_active;

                if pending.is_empty() && active_reps.is_empty() {
                    travel_phase = false;
                }
            } else if let Some(group) = wipe_groups.pop_front() {
                for id in group {
                    terminal.set_character_visibility(id, true);
                    terminal.get_characters_mut()[id as usize]
                        .animation
                        .activate_scene("brighten_scn");
                    active_ids.insert(id);
                }
            }

            if !travel_phase
                && pending.is_empty()
                && active_reps.is_empty()
                && wipe_groups.is_empty()
                && active_ids.is_empty()
            {
                break;
            }

            // Tick only the active characters, mirroring the Python effect loop.
            for character in terminal.get_characters_mut() {
                if active_ids.contains(&character.character_id) {
                    character.tick();
                }
            }
            frames.push(terminal.render_frame());

            // Retire characters with no remaining motion or animation work.
            let finished: Vec<u32> = terminal
                .get_characters()
                .iter()
                .filter(|c| active_ids.contains(&c.character_id) && !c.is_active())
                .map(|c| c.character_id)
                .collect();
            for id in finished {
                active_ids.remove(&id);
            }

            frame_count += 1;
            if frame_count >= max_frames {
                break;
            }
        }

        frames
    }
}
