//! Swarm: characters are grouped into swarms that chaotically move between
//! random focus areas on the canvas before settling into their input
//! coordinates. Port of terminaltexteffects/effects/effect_swarm.py, adapted
//! to this engine (path chaining and flash scenes are driven from the effect
//! loop instead of event handlers).

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_coords_on_circle, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

const BASE_COLOR: &str = "31a0d4";
const FLASH_COLOR: &str = "f2ea79";
const FINAL_GRADIENT_STOPS: [&str; 3] = ["8A008A", "00D1FF", "FFFFFF"];
const FINAL_GRADIENT_STEPS: usize = 12;
const SWARM_SIZE: f64 = 0.1;
const SWARM_AREA_COUNT_MIN: i32 = 2;
const SWARM_AREA_COUNT_MAX: i32 = 4;
const WAYPOINTS_PER_AREA: usize = 3;
const SWARM_START_STAGGER: usize = 25;
const MAX_TICKS: usize = 4000;

/// Small deterministic PRNG (splitmix64-flavored) so runs are reproducible
/// for a given input. The crate has no rng module, so it lives here.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut x = self.0;
        x = (x ^ (x >> 33)).wrapping_mul(0xff51afd7ed558ccd);
        ((x ^ (x >> 33)) >> 32) as u32
    }

    /// Inclusive range [lo, hi].
    fn gen_range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u32;
        lo + (self.next_u32() % span) as i32
    }

    fn gen_f64(&mut self) -> f64 {
        self.next_u32() as f64 / u32::MAX as f64
    }

    fn shuffle<T>(&mut self, v: &mut [T]) {
        if v.len() < 2 {
            return;
        }
        for i in (1..v.len()).rev() {
            let j = self.gen_range(0, i as i32) as usize;
            v.swap(i, j);
        }
    }
}

/// Per-character schedule: which paths to run, in order, and when to start.
#[derive(Clone)]
struct Plan {
    start_tick: usize,
    stages: Vec<String>,
    next_stage: usize,
    started: bool,
}

fn clamp_to_canvas(coord: Coord, width: i32, height: i32) -> Coord {
    Coord::new(coord.column.clamp(1, width), coord.row.clamp(1, height))
}

pub struct Swarm;

impl Swarm {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Swarm {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Swarm {
    fn name(&self) -> &str {
        "swarm"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;

        let seed = input
            .bytes()
            .fold(0xcbf29ce484222325u64, |acc, b| (acc ^ b as u64).wrapping_mul(0x100000001b3));
        let mut rng = Rng::new(seed);

        let base_color = Color::from_hex(BASE_COLOR).expect("valid base color");
        let flash_color = Color::from_hex(FLASH_COLOR).expect("valid flash color");
        let final_stops: Vec<Color> = FINAL_GRADIENT_STOPS
            .iter()
            .map(|hex| Color::from_hex(hex).expect("valid gradient stop"))
            .collect();
        let final_gradient = Gradient::new(&final_stops, FINAL_GRADIENT_STEPS);
        let swarm_gradient = Gradient::new(&[base_color, flash_color], 7);

        let n = terminal.get_characters().len();
        if n == 0 {
            return vec![terminal.render_frame()];
        }

        let mut plans: Vec<Plan> = vec![
            Plan {
                start_tick: 0,
                stages: Vec::new(),
                next_stage: 0,
                started: false,
            };
            n
        ];

        // Group shuffled characters into swarms.
        let mut indices: Vec<usize> = (0..n).collect();
        rng.shuffle(&mut indices);
        let swarm_char_count = ((n as f64 * SWARM_SIZE).round() as usize).max(1);
        let max_radius = (width.min(height) / 4).max(2);

        {
            let characters = terminal.get_characters_mut();
            for (swarm_index, swarm) in indices.chunks(swarm_char_count).enumerate() {
                // Swarm spawns just outside the canvas.
                let spawn = match rng.gen_range(0, 3) {
                    0 => Coord::new(-3, rng.gen_range(1, height)),
                    1 => Coord::new(width + 3, rng.gen_range(1, height)),
                    2 => Coord::new(rng.gen_range(1, width), height + 3),
                    _ => Coord::new(rng.gen_range(1, width), -3),
                };

                // Random focus areas the swarm visits before dispersing.
                let area_count = rng.gen_range(SWARM_AREA_COUNT_MIN, SWARM_AREA_COUNT_MAX);
                let area_origins: Vec<Coord> = (0..area_count)
                    .map(|_| Coord::new(rng.gen_range(1, width), rng.gen_range(1, height)))
                    .collect();

                for &char_index in swarm {
                    let character = &mut characters[char_index];
                    let symbol = character.input_symbol;
                    let input_coord = character.input_coord;
                    character.motion.current_coord = spawn;

                    let mut stages: Vec<String> = Vec::new();

                    // One path per swarm area, wandering around its origin.
                    for (area_index, origin) in area_origins.iter().enumerate() {
                        let path_id = area_index.to_string();
                        let speed = 0.3 + rng.gen_f64() * 0.4;
                        let radius_picks: Vec<Coord> = (0..WAYPOINTS_PER_AREA)
                            .map(|_| {
                                let radius = rng.gen_range(1, max_radius);
                                let ring = find_coords_on_circle(*origin, radius, 16);
                                let pick = ring[rng.gen_range(0, ring.len() as i32 - 1) as usize];
                                clamp_to_canvas(pick, width, height)
                            })
                            .collect();
                        let path = character
                            .motion
                            .new_path(&path_id, speed, Some(easing::out_sine));
                        for (k, coord) in radius_picks.iter().enumerate() {
                            path.new_waypoint(&k.to_string(), *coord);
                        }
                        stages.push(path_id);
                    }

                    // Final path home to the input coordinate.
                    let input_speed = 0.3 + rng.gen_f64() * 0.15;
                    let input_path = character
                        .motion
                        .new_path("input", input_speed, Some(easing::in_out_quad));
                    input_path.new_waypoint("input", input_coord);
                    stages.push("input".to_string());

                    // Looping swarm scene: base color pulsing toward the flash color.
                    let swarm_scene = character.animation.new_scene("swarm", true);
                    for color in swarm_gradient.spectrum.iter() {
                        swarm_scene.add_frame(symbol, 2, Some(ColorPair::fg_only(*color)));
                    }
                    for color in swarm_gradient.spectrum.iter().rev().skip(1) {
                        swarm_scene.add_frame(symbol, 2, Some(ColorPair::fg_only(*color)));
                    }

                    // Final scene: settle on the final gradient color (horizontal).
                    let fraction = if width > 1 {
                        (input_coord.column - 1) as f64 / (width - 1) as f64
                    } else {
                        0.0
                    };
                    let final_color = final_gradient
                        .get_color_at_fraction(fraction)
                        .unwrap_or(Color::new(255, 255, 255));
                    let final_scene = character.animation.new_scene("final", false);
                    final_scene.add_frame(symbol, 1, Some(ColorPair::fg_only(final_color)));

                    plans[char_index] = Plan {
                        start_tick: swarm_index * SWARM_START_STAGGER,
                        stages,
                        next_stage: 0,
                        started: false,
                    };
                }
            }
        }

        // Drive the simulation: swarms start staggered; each character chains
        // through its area paths and finally the "input" path home.
        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());
        let mut tick: usize = 0;

        loop {
            let mut any_pending = false;
            {
                let characters = terminal.get_characters_mut();
                for (index, plan) in plans.iter_mut().enumerate() {
                    let character = &mut characters[index];
                    if plan.stages.is_empty() {
                        continue;
                    }
                    if !plan.started {
                        if tick >= plan.start_tick {
                            plan.started = true;
                            character.is_visible = true;
                            character.animation.activate_scene("swarm");
                            let first = plan.stages[0].clone();
                            character.motion.activate_path(&first);
                            plan.next_stage = 1;
                        } else {
                            any_pending = true;
                        }
                    } else if plan.next_stage < plan.stages.len()
                        && character.motion.movement_is_complete()
                    {
                        let stage_id = plan.stages[plan.next_stage].clone();
                        if stage_id == "input" {
                            // Dispersing home: drop the swarm pulse, take the
                            // final gradient color.
                            character.animation.activate_scene("final");
                        }
                        character.motion.activate_path(&stage_id);
                        plan.next_stage += 1;
                    }
                    if plan.next_stage < plan.stages.len() {
                        any_pending = true;
                    }
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());
            tick += 1;

            if tick >= MAX_TICKS || (!any_pending && !terminal.is_active()) {
                break;
            }
        }

        frames
    }
}
