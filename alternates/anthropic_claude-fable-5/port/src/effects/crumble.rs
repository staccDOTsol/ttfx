//! Crumble: characters weaken (fading through a gradient), crumble and fall
//! to the canvas bottom as dust, are vacuumed into a ball at the canvas
//! center, then flung back home while strengthening to their final color.
//!
//! Port of terminaltexteffects/effects/effect_crumble.py, adapted to the
//! simplified engine (no event handlers; the effect drives phases directly).

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Deterministic little PRNG (replaces Python's `random`) so frames are
/// reproducible without pulling in an external crate.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / u32::MAX as f64
    }

    fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    fn range_usize(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u32() as usize) % n
        }
    }
}

/// Multiply a color's channels by `factor` (mirrors
/// `Animation.adjust_color_brightness` used by the Python effect to derive
/// the weakened/dust colors from the final gradient color).
fn adjust_brightness(color: Color, factor: f64) -> Color {
    let f = |v: u8| ((v as f64) * factor).round().clamp(0.0, 255.0) as u8;
    Color::new(f(color.r), f(color.g), f(color.b))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Weakening,
    Falling,
    Dust,
    Gathering,
    Ball,
    Returning,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Crumble,
    Gather,
    Hold,
    Return,
    End,
}

pub struct Crumble;

impl Crumble {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Crumble {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Crumble {
    fn name(&self) -> &str {
        "crumble"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        let center = terminal.canvas.center();
        let mut rng = Lcg::new(0xC0FF_EE00_5EED);

        // Final gradient (upstream defaults: 8A008A -> 00D1FF -> FFFFFF,
        // applied diagonally across the canvas).
        let stops = [
            Color::from_hex("8A008A").expect("valid hex"),
            Color::from_hex("00D1FF").expect("valid hex"),
            Color::from_hex("FFFFFF").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&stops, 12);

        let n = terminal.get_characters().len();
        if n == 0 {
            return vec![terminal.render_frame()];
        }
        let mut states = vec![State::Idle; n];

        // Per-character setup: scenes, paths, initial styled appearance.
        {
            let denom = ((width - 1) + (height - 1)).max(1) as f64;
            for ch in terminal.get_characters_mut().iter_mut() {
                let frac =
                    ((ch.input_coord.column - 1) + (ch.input_coord.row - 1)) as f64 / denom;
                let final_color = final_gradient
                    .get_color_at_fraction(frac)
                    .unwrap_or(stops[2]);
                let weak_color = adjust_brightness(final_color, 0.65);
                let dust_color = adjust_brightness(final_color, 0.55);

                // "weaken" scene: fade from the final color down to dust.
                {
                    let weaken_gradient =
                        Gradient::new(&[final_color, weak_color, dust_color], 3);
                    let scene = ch.animation.new_scene("weaken", false);
                    for color in &weaken_gradient.spectrum {
                        scene.add_frame(ch.input_symbol, 3, Some(ColorPair::fg_only(*color)));
                    }
                }

                // "strengthen" scene: bright flash settling on the final color.
                {
                    let white = Color::new(0xFF, 0xFF, 0xFF);
                    let strengthen_gradient = Gradient::new(&[white, final_color], 6);
                    let scene = ch.animation.new_scene("strengthen", false);
                    for color in &strengthen_gradient.spectrum {
                        scene.add_frame(ch.input_symbol, 3, Some(ColorPair::fg_only(*color)));
                    }
                }

                // "fall" path: drop straight down with a bounce.
                let fall_speed = rng.range_f64(0.2, 0.45);
                let fall = ch.motion.new_path("fall", fall_speed, Some(easing::out_bounce));
                fall.new_waypoint("bottom", Coord::new(ch.input_coord.column, 1));

                // "gather" path: vacuumed toward the canvas center.
                let gather_speed = rng.range_f64(0.3, 0.5);
                let gather = ch.motion.new_path("gather", gather_speed, Some(easing::in_expo));
                gather.new_waypoint("center", center);

                // "input" path: flung back to the home coordinate.
                let home = ch.motion.new_path("input", 0.4, Some(easing::out_cubic));
                home.new_waypoint("home", ch.input_coord);

                ch.animation
                    .set_appearance(ch.input_symbol, Some(ColorPair::fg_only(final_color)));
                ch.is_visible = true;
            }
        }

        // Crumble order is shuffled; the vacuum sweeps left-to-right.
        let mut crumble_order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = rng.range_usize(i + 1);
            crumble_order.swap(i, j);
        }
        let mut gather_order: Vec<usize> = (0..n).collect();
        gather_order.sort_by_key(|&i| {
            let c = &terminal.get_characters()[i];
            (c.input_coord.column, c.input_coord.row)
        });
        let return_order = gather_order.clone();

        let weaken_per_tick = (n / 40).max(1);
        let gather_per_tick = (n / 25).max(1);
        let return_per_tick = (n / 25).max(2);

        let mut phase = Phase::Crumble;
        let mut next_weaken = 0usize;
        let mut next_gather = 0usize;
        let mut next_return = 0usize;
        let mut hold_ticks: u32 = 15;

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());
        let max_frames = 3000usize;

        while phase != Phase::End && frames.len() < max_frames {
            match phase {
                Phase::Crumble => {
                    // Stagger weaken activations.
                    let mut launched = 0usize;
                    while next_weaken < n && launched < weaken_per_tick {
                        let idx = crumble_order[next_weaken];
                        let ch = &mut terminal.get_characters_mut()[idx];
                        ch.animation.activate_scene("weaken");
                        states[idx] = State::Weakening;
                        next_weaken += 1;
                        launched += 1;
                    }
                    // Weakened characters crumble and fall.
                    for idx in 0..n {
                        match states[idx] {
                            State::Weakening => {
                                let ch = &mut terminal.get_characters_mut()[idx];
                                if ch.animation.active_scene_is_complete() {
                                    ch.motion.activate_path("fall");
                                    states[idx] = State::Falling;
                                }
                            }
                            State::Falling => {
                                let ch = &terminal.get_characters()[idx];
                                if ch.motion.movement_is_complete() {
                                    states[idx] = State::Dust;
                                }
                            }
                            _ => {}
                        }
                    }
                    if states.iter().all(|s| *s == State::Dust) {
                        phase = Phase::Gather;
                    }
                }
                Phase::Gather => {
                    // The vacuum sweeps across, pulling dust to the center.
                    let mut launched = 0usize;
                    while next_gather < n && launched < gather_per_tick {
                        let idx = gather_order[next_gather];
                        let ch = &mut terminal.get_characters_mut()[idx];
                        ch.motion.activate_path("gather");
                        states[idx] = State::Gathering;
                        next_gather += 1;
                        launched += 1;
                    }
                    for idx in 0..n {
                        if states[idx] == State::Gathering {
                            let ch = &terminal.get_characters()[idx];
                            if ch.motion.movement_is_complete() {
                                states[idx] = State::Ball;
                            }
                        }
                    }
                    if states.iter().all(|s| *s == State::Ball) {
                        phase = Phase::Hold;
                    }
                }
                Phase::Hold => {
                    if hold_ticks > 0 {
                        hold_ticks -= 1;
                    } else {
                        phase = Phase::Return;
                    }
                }
                Phase::Return => {
                    let mut launched = 0usize;
                    while next_return < n && launched < return_per_tick {
                        let idx = return_order[next_return];
                        let ch = &mut terminal.get_characters_mut()[idx];
                        ch.motion.activate_path("input");
                        ch.animation.activate_scene("strengthen");
                        states[idx] = State::Returning;
                        next_return += 1;
                        launched += 1;
                    }
                    for idx in 0..n {
                        if states[idx] == State::Returning {
                            let ch = &terminal.get_characters()[idx];
                            if ch.motion.movement_is_complete()
                                && ch.animation.active_scene_is_complete()
                            {
                                states[idx] = State::Done;
                            }
                        }
                    }
                    if states.iter().all(|s| *s == State::Done) {
                        phase = Phase::End;
                    }
                }
                Phase::End => {}
            }

            if phase == Phase::End {
                break;
            }
            terminal.tick();
            frames.push(terminal.render_frame());
        }

        // Final settled frame with every character home and fully colored.
        frames.push(terminal.render_frame());
        frames
    }
}
