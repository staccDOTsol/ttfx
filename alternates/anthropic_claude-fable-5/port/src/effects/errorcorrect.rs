//! Errorcorrect effect: swapped character pairs are flagged as errors,
//! flash in the error color, then swap back to their correct positions,
//! wiping through the correction gradient before settling into the final
//! gradient color.
//!
//! Port of terminaltexteffects/effects/effect_errorcorrect.py.

use std::collections::{HashMap, VecDeque};

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

const ERROR_COLOR: &str = "e74c3c";
const CORRECT_COLOR: &str = "45bf55";
const FINAL_GRADIENT_STOPS: [&str; 3] = ["8A008A", "00D1FF", "FFFFFF"];
const ERROR_PAIRS: f64 = 0.1;
const SWAP_DELAY: u32 = 10;
const MOVEMENT_SPEED: f64 = 0.5;
const MAX_FRAMES: usize = 5000;

const BLOCK_WIPE_START: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const BLOCK_WIPE_END: [char; 7] = ['▇', '▆', '▅', '▄', '▃', '▂', '▁'];

/// Per-swapped-character progress through the correction sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Waiting,
    Error,
    Moving,
    Final,
    Done,
}

/// Small deterministic xorshift PRNG (the crate has no rand dependency).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn gen_range(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }
}

pub struct Errorcorrect;

impl Errorcorrect {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Errorcorrect {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Errorcorrect {
    fn name(&self) -> &str {
        "errorcorrect"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let height = terminal.canvas.height;

        let error_color = Color::from_hex(ERROR_COLOR).expect("valid hex");
        let correct_color = Color::from_hex(CORRECT_COLOR).expect("valid hex");
        let white = Color::new(255, 255, 255);
        let stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .map(|s| Color::from_hex(s).expect("valid hex"))
            .collect();
        let final_gradient = Gradient::new(&stops, 12);
        let correcting_gradient = Gradient::new(&[error_color, correct_color], 10);

        // Snapshot character ids and home coordinates.
        let infos: Vec<(u32, Coord)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.character_id, c.input_coord))
            .collect();

        // Final gradient color per character (vertical direction, as upstream).
        let mut final_colors: HashMap<u32, Color> = HashMap::new();
        for (id, coord) in &infos {
            let t = if height > 1 {
                (coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let color = final_gradient.get_color_at_fraction(t).unwrap_or(correct_color);
            final_colors.insert(*id, color);
        }

        // Select random pairs of characters to swap.
        let coord_of: HashMap<u32, Coord> = infos.iter().copied().collect();
        let mut rng = Rng::new(0x5EED_1234_ABCD_EF01);
        let mut ids: Vec<u32> = infos.iter().map(|(id, _)| *id).collect();
        let total = ids.len();
        let number_of_swaps = if total >= 2 {
            ((total as f64 * ERROR_PAIRS) as usize).max(1)
        } else {
            0
        };
        let mut pairs: Vec<(u32, u32)> = Vec::new();
        for _ in 0..number_of_swaps {
            if ids.len() < 2 {
                break;
            }
            let a = ids.remove(rng.gen_range(ids.len()));
            let b = ids.remove(rng.gen_range(ids.len()));
            pairs.push((a, b));
        }

        // Each swapped character starts at its partner's home coordinate.
        let mut swap_start: HashMap<u32, Coord> = HashMap::new();
        for (a, b) in &pairs {
            swap_start.insert(*a, coord_of[b]);
            swap_start.insert(*b, coord_of[a]);
        }

        // Configure every character: visible, styled with its final color;
        // swapped characters get error/correcting/final scenes and a return path.
        for ch in terminal.get_characters_mut() {
            ch.is_visible = true;
            let final_color = final_colors[&ch.character_id];
            if let Some(&start) = swap_start.get(&ch.character_id) {
                ch.motion.current_coord = start;
                let symbol = ch.input_symbol;
                ch.animation
                    .set_appearance(symbol, Some(ColorPair::fg_only(error_color)));

                // Error scene: block-wipe in, then flash between inverted
                // symbol and error-colored shade block.
                {
                    let scene = ch.animation.new_scene("error", false);
                    for block in BLOCK_WIPE_START {
                        scene.add_frame(block, 3, Some(ColorPair::fg_only(error_color)));
                    }
                    for _ in 0..10 {
                        scene.add_frame(
                            symbol,
                            3,
                            Some(ColorPair::new(Some(white), Some(error_color))),
                        );
                        scene.add_frame('▓', 3, Some(ColorPair::fg_only(error_color)));
                    }
                }

                // Correcting scene: full block wiping from error to correct color.
                {
                    let scene = ch.animation.new_scene("correcting", false);
                    for color in &correcting_gradient.spectrum {
                        scene.add_frame('█', 3, Some(ColorPair::fg_only(*color)));
                    }
                }

                // Final scene: block-wipe out, then settle into the final
                // gradient color on the input symbol.
                {
                    let settle = Gradient::new(&[correct_color, final_color], 10);
                    let scene = ch.animation.new_scene("final", false);
                    for block in BLOCK_WIPE_END {
                        scene.add_frame(block, 3, Some(ColorPair::fg_only(correct_color)));
                    }
                    for color in &settle.spectrum {
                        scene.add_frame(symbol, 3, Some(ColorPair::fg_only(*color)));
                    }
                }

                let home = ch.input_coord;
                let path = ch.motion.new_path("input_coord", MOVEMENT_SPEED, None);
                path.new_waypoint("input_coord", home);
            } else {
                ch.animation
                    .set_appearance(ch.input_symbol, Some(ColorPair::fg_only(final_color)));
            }
        }

        // Frame loop: launch one pair every SWAP_DELAY ticks, then walk each
        // swapped character through error -> correcting/moving -> final.
        let mut states: HashMap<u32, State> = swap_start
            .keys()
            .map(|&id| (id, State::Waiting))
            .collect();
        let mut pending: VecDeque<(u32, u32)> = pairs.into_iter().collect();
        let mut launch_countdown: u32 = 0;

        let mut frames = Vec::new();
        frames.push(terminal.render_frame());

        let mut frame_count = 0usize;
        loop {
            let done = pending.is_empty()
                && states.values().all(|s| *s == State::Done)
                && !terminal.is_active();
            if done || frame_count >= MAX_FRAMES {
                break;
            }

            // Launch the next swapped pair.
            if launch_countdown == 0 {
                if let Some((a, b)) = pending.pop_front() {
                    for id in [a, b] {
                        for ch in terminal.get_characters_mut() {
                            if ch.character_id == id {
                                ch.animation.activate_scene("error");
                            }
                        }
                        states.insert(id, State::Error);
                    }
                    launch_countdown = SWAP_DELAY;
                }
            } else {
                launch_countdown -= 1;
            }

            // Advance per-character state machines.
            let active_ids: Vec<u32> = states.keys().copied().collect();
            for id in active_ids {
                let state = states[&id];
                for ch in terminal.get_characters_mut() {
                    if ch.character_id != id {
                        continue;
                    }
                    match state {
                        State::Error => {
                            if ch.animation.active_scene_is_complete() {
                                ch.animation.activate_scene("correcting");
                                ch.motion.activate_path("input_coord");
                                states.insert(id, State::Moving);
                            }
                        }
                        State::Moving => {
                            if ch.motion.movement_is_complete() {
                                ch.animation.activate_scene("final");
                                states.insert(id, State::Final);
                            }
                        }
                        State::Final => {
                            if ch.animation.active_scene_is_complete() {
                                states.insert(id, State::Done);
                            }
                        }
                        State::Waiting | State::Done => {}
                    }
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());
            frame_count += 1;
        }

        frames
    }
}
