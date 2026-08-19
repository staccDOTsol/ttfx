//! Bubbles effect: characters are grouped into bubbles that float down the
//! canvas and pop when they reach the bottom, scattering the characters
//! before they settle (colored by the final gradient) at their input coords.
//!
//! Port of terminaltexteffects/effects/effect_bubbles.py.

use super::Effect;
use crate::engine::motion::Path;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

const BUBBLE_COLORS: [&str; 4] = ["d33aff", "7395c4", "43c2a7", "02ff7f"];
const POP_COLOR: &str = "ffffff";
const FINAL_GRADIENT_STOPS: [&str; 3] = ["d33aff", "02ff7f", "ffffff"];
const BUBBLE_SPEED: f64 = 0.25;
const BUBBLE_DELAY: usize = 15;
const POP_OUT_SPEED: f64 = 0.3;
const MOVEMENT_SPEED: f64 = 0.25;
const MAX_FRAMES: usize = 2000;

/// Tiny deterministic xorshift PRNG (no external rand dependency).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9e3779b97f4a7c15 } else { seed })
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    /// Inclusive range [lo, hi].
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() % ((hi - lo + 1) as u32)) as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharState {
    InBubble,
    PopOut,
    Final,
    Done,
}

/// One bubble: a set of member characters arranged on a circle around an
/// invisible anchor point that floats down to the bubble's landing row.
struct Bubble {
    member_indices: Vec<usize>,
    anchor_path: Path,
    anchor_coord: Coord,
    radius: i32,
    activation_tick: usize,
    active: bool,
    landed: bool,
}

pub struct Bubbles;

impl Bubbles {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Bubbles {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Bubbles {
    fn name(&self) -> &str {
        "bubbles"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;

        if terminal.get_characters().is_empty() {
            return vec![terminal.render_frame()];
        }

        let mut rng = Rng::new(0x5eed_b0bb_1e50_0001);

        let pop_color = Color::from_hex(POP_COLOR).expect("valid pop color");
        let bubble_palette: Vec<Color> = BUBBLE_COLORS
            .iter()
            .map(|h| Color::from_hex(h).expect("valid bubble color"))
            .collect();
        let gradient_stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .map(|h| Color::from_hex(h).expect("valid gradient stop"))
            .collect();
        let final_gradient = Gradient::new(&gradient_stops, 12);

        // --- per-character setup: pop scene, final scene, final path ---
        {
            let chars = terminal.get_characters_mut();
            for ch in chars.iter_mut() {
                let sym = ch.input_symbol;
                let t = if height > 1 {
                    (ch.input_coord.row - 1) as f64 / (height - 1) as f64
                } else {
                    0.0
                };
                let final_color = final_gradient
                    .get_color_at_fraction(t)
                    .unwrap_or(Color::new(255, 255, 255));

                let pop_scn = ch.animation.new_scene("pop", false);
                pop_scn.add_frame('*', 5, Some(ColorPair::fg_only(pop_color)));
                pop_scn.add_frame('\'', 5, Some(ColorPair::fg_only(pop_color)));
                pop_scn.add_frame('.', 5, Some(ColorPair::fg_only(pop_color)));

                let final_scn = ch.animation.new_scene("final", false);
                final_scn.add_frame(sym, 1, Some(ColorPair::fg_only(final_color)));

                let final_path =
                    ch.motion
                        .new_path("final", MOVEMENT_SPEED, Some(easing::in_out_sine));
                final_path.new_waypoint("input_coord", ch.input_coord);
            }
        }

        // --- group characters into bubbles (rows top to bottom, left to right) ---
        let char_count = terminal.get_characters().len();
        let mut order: Vec<usize> = (0..char_count).collect();
        {
            let chars = terminal.get_characters();
            order.sort_by_key(|&i| (-(chars[i].input_coord.row), chars[i].input_coord.column));
        }

        let mut bubbles: Vec<Bubble> = Vec::new();
        let mut idx = 0usize;
        while idx < order.len() {
            let remaining = order.len() - idx;
            let size = if remaining < 5 {
                remaining
            } else {
                rng.range(5, remaining.min(15) as i32) as usize
            };
            let members: Vec<usize> = order[idx..idx + size].to_vec();
            idx += size;

            let radius = ((members.len() as i32 + 4) / 5)
                .max(1)
                .min((width.min(height) / 2).max(1));
            let col_lo = (1 + radius).min(width);
            let col_hi = (width - radius).max(col_lo);
            let origin = Coord::new(rng.range(col_lo, col_hi), (height - radius).max(1));
            let floor_coord = Coord::new(rng.range(col_lo, col_hi), (radius + 1).min(height));

            let mut anchor_path = Path::new("floor", BUBBLE_SPEED, None);
            anchor_path.new_waypoint("floor", floor_coord);
            anchor_path.activate(origin);

            // sheen scene: bubble color while floating
            let bubble_color = bubble_palette[(rng.next_u32() as usize) % bubble_palette.len()];
            {
                let chars = terminal.get_characters_mut();
                for &mi in &members {
                    let ch = &mut chars[mi];
                    let sym = ch.input_symbol;
                    let sheen = ch.animation.new_scene("sheen", true);
                    sheen.add_frame(sym, 1, Some(ColorPair::fg_only(bubble_color)));
                }
            }

            let activation_tick = bubbles.len() * BUBBLE_DELAY;
            bubbles.push(Bubble {
                member_indices: members,
                anchor_path,
                anchor_coord: origin,
                radius,
                activation_tick,
                active: false,
                landed: false,
            });
        }

        // --- frame loop ---
        let mut states = vec![CharState::InBubble; char_count];
        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());
        let mut tick_count: usize = 0;

        loop {
            // activate bubbles once their delay has elapsed
            for bubble in bubbles.iter_mut() {
                if !bubble.active && tick_count >= bubble.activation_tick {
                    bubble.active = true;
                    let chars = terminal.get_characters_mut();
                    let coords = find_coords_on_circle(
                        bubble.anchor_coord,
                        bubble.radius,
                        bubble.member_indices.len(),
                    );
                    for (k, &mi) in bubble.member_indices.iter().enumerate() {
                        chars[mi].is_visible = true;
                        chars[mi].animation.activate_scene("sheen");
                        chars[mi].motion.current_coord = coords[k];
                    }
                }
            }

            // move anchors and keep members on the bubble circle; pop on landing
            for bubble in bubbles.iter_mut() {
                if !bubble.active || bubble.landed {
                    continue;
                }
                bubble.anchor_coord = bubble.anchor_path.step();
                let coords = find_coords_on_circle(
                    bubble.anchor_coord,
                    bubble.radius,
                    bubble.member_indices.len(),
                );
                {
                    let chars = terminal.get_characters_mut();
                    for (k, &mi) in bubble.member_indices.iter().enumerate() {
                        chars[mi].motion.current_coord = coords[k];
                    }
                }
                if bubble.anchor_path.is_complete() {
                    bubble.landed = true;
                    // pop: scatter members outward, then send them home
                    let pop_coords = find_coords_on_circle(
                        bubble.anchor_coord,
                        bubble.radius + 3,
                        bubble.member_indices.len(),
                    );
                    let chars = terminal.get_characters_mut();
                    for (k, &mi) in bubble.member_indices.iter().enumerate() {
                        let ch = &mut chars[mi];
                        let pop_out =
                            ch.motion
                                .new_path("pop_out", POP_OUT_SPEED, Some(easing::out_expo));
                        pop_out.new_waypoint("pop_out", pop_coords[k]);
                        ch.motion.activate_path("pop_out");
                        ch.animation.activate_scene("pop");
                        states[mi] = CharState::PopOut;
                    }
                }
            }

            // advance all characters one tick
            terminal.tick();

            // state transitions after movement
            {
                let chars = terminal.get_characters_mut();
                for i in 0..chars.len() {
                    match states[i] {
                        CharState::PopOut => {
                            if chars[i].motion.movement_is_complete() {
                                chars[i].motion.activate_path("final");
                                chars[i].animation.activate_scene("final");
                                states[i] = CharState::Final;
                            }
                        }
                        CharState::Final => {
                            if chars[i].motion.movement_is_complete() {
                                states[i] = CharState::Done;
                            }
                        }
                        _ => {}
                    }
                }
            }

            frames.push(terminal.render_frame());
            tick_count += 1;

            let all_done = states.iter().all(|s| *s == CharState::Done);
            if all_done || tick_count >= MAX_FRAMES {
                break;
            }
        }

        frames
    }
}
