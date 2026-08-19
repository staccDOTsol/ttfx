//! Fireworks: characters are gathered into shells, launched from the bottom of the
//! canvas, exploded outward on a circle, then fall to their input coordinates while
//! fading from the shell color to their final gradient color.
//! Port of terminaltexteffects/effects/effect_fireworks.py adapted to this engine's
//! explicit state-machine driving (no event handlers).

use std::collections::HashMap;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

const FIREWORK_COLORS: [&str; 5] = ["88F7E2", "44D492", "F5EB67", "FFA15C", "FA233E"];
const FINAL_GRADIENT_STOPS: [&str; 3] = ["8A008A", "00D1FF", "FFFFFF"];
const FIREWORK_SYMBOL: char = 'o';
const FIREWORK_VOLUME: f64 = 0.02;
const LAUNCH_DELAY: usize = 30;
const EXPLODE_DISTANCE: f64 = 0.1;
const LAUNCH_SPEED: f64 = 0.2;
const EXPLODE_SPEED: f64 = 0.3;
const FALL_SPEED: f64 = 0.4;
const MAX_FRAMES: usize = 5000;

/// Small deterministic xorshift PRNG so the effect needs no external crates.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Inclusive integer range; returns `lo` when the range is degenerate.
    fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i32
    }

    fn choice_hex(&mut self, items: &[&'static str]) -> &'static str {
        items[(self.next_u64() % items.len() as u64) as usize]
    }

    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            v.swap(i, j);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Waiting,
    Launching,
    Exploding,
    Falling,
    Done,
}

struct CharState {
    phase: Phase,
    launch_tick: usize,
    launch_coord: Coord,
    shell_color: Color,
    final_color: Color,
    fall_gradient: Gradient,
}

pub struct Fireworks;

impl Fireworks {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Fireworks {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Fireworks {
    fn name(&self) -> &str {
        "fireworks"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        let mut rng = Rng::new(0x5eed_f14e_u64 ^ ((input.len() as u64) << 7).wrapping_add(0x9e37_79b9));

        let final_stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .map(|hex| Color::from_hex(hex).expect("valid hex"))
            .collect();
        let final_gradient = Gradient::new(&final_stops, height.max(2) as usize);
        let fallback_final = *final_stops.last().expect("non-empty stops");

        let mut ids: Vec<u32> = terminal
            .get_characters()
            .iter()
            .map(|c| c.character_id)
            .collect();
        let total = ids.len();
        if total == 0 {
            return vec![terminal.render_frame()];
        }
        rng.shuffle(&mut ids);

        let shell_size = ((FIREWORK_VOLUME * total as f64).round() as usize).max(1);
        let radius = ((EXPLODE_DISTANCE * width as f64).round() as i32).max(2);

        let mut states: HashMap<u32, CharState> = HashMap::new();

        for (shell_index, shell) in ids.chunks(shell_size).enumerate() {
            // Pick an apex for this shell, kept inside the canvas with room to burst.
            let col_lo = (1 + radius).min(width);
            let col_hi = (width - radius).max(col_lo);
            let row_lo = (height / 2).max(1);
            let row_hi = (height - radius).max(row_lo);
            let apex = Coord::new(rng.range_i32(col_lo, col_hi), rng.range_i32(row_lo, row_hi));

            let shell_color =
                Color::from_hex(rng.choice_hex(&FIREWORK_COLORS)).expect("valid hex");
            let burst_coords = find_coords_on_circle(apex, radius, shell.len());
            let launch_tick =
                shell_index * LAUNCH_DELAY + rng.range_i32(0, LAUNCH_DELAY as i32) as usize;

            for (i, &id) in shell.iter().enumerate() {
                let character = &mut terminal.get_characters_mut()[id as usize];
                let input_coord = character.input_coord;

                let launch_path =
                    character
                        .motion
                        .new_path("launch", LAUNCH_SPEED, Some(easing::out_expo));
                launch_path.new_waypoint("apex", apex);

                let explode_path =
                    character
                        .motion
                        .new_path("explode", EXPLODE_SPEED, Some(easing::out_quad));
                explode_path.new_waypoint("burst", burst_coords[i]);

                let fall_path =
                    character
                        .motion
                        .new_path("fall", FALL_SPEED, Some(easing::in_out_cubic));
                fall_path.new_waypoint("home", input_coord);

                let row_fraction = if height > 1 {
                    (input_coord.row - 1) as f64 / (height - 1) as f64
                } else {
                    0.0
                };
                let final_color = final_gradient
                    .get_color_at_fraction(row_fraction)
                    .unwrap_or(fallback_final);
                let fall_gradient = Gradient::new(&[shell_color, final_color], 12);

                states.insert(
                    id,
                    CharState {
                        phase: Phase::Waiting,
                        launch_tick,
                        launch_coord: Coord::new(apex.column, 1),
                        shell_color,
                        final_color,
                        fall_gradient,
                    },
                );
            }
        }

        let mut frames = vec![terminal.render_frame()];
        let mut tick = 0usize;

        loop {
            let mut all_done = true;

            for character in terminal.get_characters_mut() {
                let state = match states.get_mut(&character.character_id) {
                    Some(state) => state,
                    None => continue,
                };
                match state.phase {
                    Phase::Waiting => {
                        all_done = false;
                        if tick >= state.launch_tick {
                            character.motion.current_coord = state.launch_coord;
                            character.is_visible = true;
                            character.animation.set_appearance(
                                FIREWORK_SYMBOL,
                                Some(ColorPair::fg_only(state.shell_color)),
                            );
                            character.motion.activate_path("launch");
                            state.phase = Phase::Launching;
                        }
                    }
                    Phase::Launching => {
                        all_done = false;
                        if character.motion.movement_is_complete() {
                            character.motion.activate_path("explode");
                            state.phase = Phase::Exploding;
                        }
                    }
                    Phase::Exploding => {
                        all_done = false;
                        if character.motion.movement_is_complete() {
                            character.motion.activate_path("fall");
                            state.phase = Phase::Falling;
                        }
                    }
                    Phase::Falling => {
                        all_done = false;
                        if character.motion.movement_is_complete() {
                            character.animation.set_appearance(
                                character.input_symbol,
                                Some(ColorPair::fg_only(state.final_color)),
                            );
                            state.phase = Phase::Done;
                        } else {
                            let progress = character
                                .motion
                                .query_path("fall")
                                .map(|p| {
                                    if p.max_steps > 0 {
                                        p.current_step as f64 / p.max_steps as f64
                                    } else {
                                        1.0
                                    }
                                })
                                .unwrap_or(1.0);
                            let color = state
                                .fall_gradient
                                .get_color_at_fraction(progress)
                                .unwrap_or(state.final_color);
                            character.animation.set_appearance(
                                character.input_symbol,
                                Some(ColorPair::fg_only(color)),
                            );
                        }
                    }
                    Phase::Done => {}
                }
            }

            if all_done || tick >= MAX_FRAMES {
                break;
            }

            terminal.tick();
            frames.push(terminal.render_frame());
            tick += 1;
        }

        frames
    }
}
