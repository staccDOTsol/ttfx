//! Spotlights effect: the text sits in darkness while a handful of roaming
//! spotlights sweep the canvas, illuminating characters that fall inside the
//! beam (with a soft falloff at the edge). After the search phase the
//! spotlights converge on the center of the canvas, then the beam expands
//! until the entire text is fully illuminated in its final gradient colors.
//!
//! Port of terminaltexteffects/effects/effect_spotlights.py.

use std::collections::HashMap;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::easing;
use crate::utils::geometry::{find_length_of_line, Coord};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Number of spotlights roaming the canvas (Python default: 3).
const SPOTLIGHT_COUNT: usize = 3;
/// Beam radius as a divisor of the smallest canvas dimension (Python: beam_width_ratio = 2.0).
const BEAM_WIDTH_RATIO: f64 = 2.0;
/// Fraction of the beam radius over which brightness falls off (Python: beam_falloff = 0.3).
const BEAM_FALLOFF: f64 = 0.3;
/// Frames spent in the search phase before converging on the center.
const SEARCH_DURATION: u32 = 200;
/// Speed range for the search movement (Python: search_speed_range = (0.25, 0.5)).
const SEARCH_SPEED_MIN: f64 = 0.25;
const SEARCH_SPEED_MAX: f64 = 0.5;
/// Number of random search waypoint paths per spotlight (Python builds 10).
const SEARCH_PATH_COUNT: usize = 10;
/// Brightness factor applied to characters outside any beam.
const DARK_FACTOR: f64 = 0.2;
/// Hard cap on emitted frames.
const MAX_FRAMES: usize = 2000;

/// Default final gradient stops from the Python effect config.
const GRADIENT_STOPS: [&str; 3] = ["ab48ff", "e7b2b2", "fffebd"];
const GRADIENT_STEPS: usize = 12;

/// Small deterministic xorshift PRNG (no external crates available).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Inclusive integer range.
    fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo + 1) as u64;
        lo + (self.next() % span) as i32
    }

    fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        let t = self.next() as f64 / u64::MAX as f64;
        lo + (hi - lo) * t
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Search,
    Converge,
    Expand,
    Complete,
}

/// Scale a color's brightness by `factor` (0..=1), mirroring the Python
/// `Animation.adjust_color_brightness` usage in this effect.
fn adjust_brightness(color: Color, factor: f64) -> Color {
    let f = factor.clamp(0.0, 1.0);
    Color::new(
        (color.r as f64 * f).round().clamp(0.0, 255.0) as u8,
        (color.g as f64 * f).round().clamp(0.0, 255.0) as u8,
        (color.b as f64 * f).round().clamp(0.0, 255.0) as u8,
    )
}

/// Random canvas coordinate at least `min_distance` cells from `origin`.
fn find_coord_at_minimum_distance(
    rng: &mut Rng,
    origin: Coord,
    min_distance: i32,
    width: i32,
    height: i32,
) -> Coord {
    for _ in 0..1000 {
        let candidate = Coord::new(rng.range_i32(1, width), rng.range_i32(1, height));
        if find_length_of_line(origin, candidate) >= min_distance as f64 {
            return candidate;
        }
    }
    Coord::new(width, height)
}

/// Recolor every text character based on its distance to the nearest
/// spotlight: full gradient color inside the beam core, eased-down brightness
/// through the falloff band, and the darkened color outside all beams.
fn illuminate(
    terminal: &mut Terminal,
    spot_coords: &[Coord],
    radius: f64,
    color_map: &HashMap<u32, (Color, Color)>,
) {
    let edge = radius * (1.0 - BEAM_FALLOFF);
    let falloff_span = (radius * BEAM_FALLOFF).max(f64::EPSILON);
    for character in terminal.get_characters_mut() {
        let Some(&(bright, dark)) = color_map.get(&character.character_id) else {
            continue; // spotlight anchors have no colors and stay invisible
        };
        let mut distance = f64::MAX;
        for coord in spot_coords {
            distance = distance.min(find_length_of_line(character.motion.current_coord, *coord));
        }
        let color = if !spot_coords.is_empty() && distance <= radius {
            if distance > edge {
                let factor = (1.0 - (distance - edge) / falloff_span).max(DARK_FACTOR);
                adjust_brightness(bright, factor)
            } else {
                bright
            }
        } else {
            dark
        };
        character
            .animation
            .set_appearance(character.input_symbol, Some(ColorPair::fg_only(color)));
    }
}

pub struct Spotlights;

impl Spotlights {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Spotlights {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Spotlights {
    fn name(&self) -> &str {
        "spotlights"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.config.width;
        let height = terminal.config.height;
        let center = terminal.canvas.center();
        let mut rng = Rng::new(0x5EED_C0DE_u64);

        // Final gradient, mapped vertically across the canvas.
        let stops: Vec<Color> = GRADIENT_STOPS
            .iter()
            .filter_map(|hex| Color::from_hex(hex))
            .collect();
        let gradient = Gradient::new(&stops, GRADIENT_STEPS);

        // Per-character (bright, dark) colors, keyed by character id.
        let mut color_map: HashMap<u32, (Color, Color)> = HashMap::new();
        for character in terminal.get_characters() {
            let fraction = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let bright = gradient
                .get_color_at_fraction(fraction)
                .unwrap_or(Color::new(255, 255, 255));
            let dark = adjust_brightness(bright, DARK_FACTOR);
            color_map.insert(character.character_id, (bright, dark));
        }

        // All text characters start visible but darkened.
        for character in terminal.get_characters_mut() {
            if let Some(&(_, dark)) = color_map.get(&character.character_id) {
                character.is_visible = true;
                character
                    .animation
                    .set_appearance(character.input_symbol, Some(ColorPair::fg_only(dark)));
            }
        }

        // Build the spotlights: invisible anchor characters with chained
        // random search paths plus a final "center" path.
        let min_distance = (width.max(height) / 4).max(1);
        let mut spotlight_ids: Vec<u32> = Vec::with_capacity(SPOTLIGHT_COUNT);
        for _ in 0..SPOTLIGHT_COUNT {
            let start = Coord::new(rng.range_i32(1, width), rng.range_i32(1, height));
            let mut targets: Vec<(Coord, f64)> = Vec::with_capacity(SEARCH_PATH_COUNT);
            let mut last = start;
            for _ in 0..SEARCH_PATH_COUNT {
                let next = find_coord_at_minimum_distance(&mut rng, last, min_distance, width, height);
                targets.push((next, rng.range_f64(SEARCH_SPEED_MIN, SEARCH_SPEED_MAX)));
                last = next;
            }
            let id = terminal.add_character('O', start);
            if let Some(spotlight) = terminal
                .get_characters_mut()
                .iter_mut()
                .find(|c| c.character_id == id)
            {
                for (index, (coord, speed)) in targets.iter().enumerate() {
                    let path = spotlight
                        .motion
                        .new_path(&index.to_string(), *speed, Some(easing::in_out_quad));
                    path.new_waypoint("", *coord);
                }
                let center_path = spotlight
                    .motion
                    .new_path("center", 0.5, Some(easing::in_out_sine));
                center_path.new_waypoint("", center);
                spotlight.motion.activate_path("0");
            }
            spotlight_ids.push(id);
        }

        // Beam geometry.
        let base_radius = ((width.min(height)) as f64 / BEAM_WIDTH_RATIO).round().max(2.0);
        let corners = [
            Coord::new(1, 1),
            Coord::new(width, 1),
            Coord::new(1, height),
            Coord::new(width, height),
        ];
        let max_radius = corners
            .iter()
            .map(|c| find_length_of_line(center, *c))
            .fold(0.0_f64, f64::max)
            + base_radius;
        let expand_rate = ((max_radius - base_radius) / 60.0).max(0.5);

        let mut frames_out: Vec<String> = Vec::new();
        let mut phase = Phase::Search;
        let mut search_remaining = SEARCH_DURATION;
        let mut path_indices = vec![0usize; spotlight_ids.len()];
        let mut radius = base_radius;

        // Initial frame: darkness plus the starting beam positions.
        let spot_coords: Vec<Coord> = spotlight_ids
            .iter()
            .filter_map(|&id| {
                terminal
                    .get_characters()
                    .iter()
                    .find(|c| c.character_id == id)
                    .map(|c| c.motion.current_coord)
            })
            .collect();
        illuminate(&mut terminal, &spot_coords, radius, &color_map);
        frames_out.push(terminal.render_frame());

        while frames_out.len() < MAX_FRAMES {
            match phase {
                Phase::Search => {
                    // Chain to the next search path when the current one completes.
                    for (i, &id) in spotlight_ids.iter().enumerate() {
                        let done = terminal
                            .get_characters()
                            .iter()
                            .find(|c| c.character_id == id)
                            .map(|c| c.motion.movement_is_complete())
                            .unwrap_or(true);
                        if done {
                            path_indices[i] = (path_indices[i] + 1) % SEARCH_PATH_COUNT;
                            let path_id = path_indices[i].to_string();
                            if let Some(spotlight) = terminal
                                .get_characters_mut()
                                .iter_mut()
                                .find(|c| c.character_id == id)
                            {
                                spotlight.motion.activate_path(&path_id);
                            }
                        }
                    }
                    search_remaining = search_remaining.saturating_sub(1);
                    if search_remaining == 0 {
                        for &id in &spotlight_ids {
                            if let Some(spotlight) = terminal
                                .get_characters_mut()
                                .iter_mut()
                                .find(|c| c.character_id == id)
                            {
                                spotlight.motion.activate_path("center");
                            }
                        }
                        phase = Phase::Converge;
                    }
                }
                Phase::Converge => {
                    let all_centered = spotlight_ids.iter().all(|&id| {
                        terminal
                            .get_characters()
                            .iter()
                            .find(|c| c.character_id == id)
                            .map(|c| c.motion.movement_is_complete())
                            .unwrap_or(true)
                    });
                    if all_centered {
                        phase = Phase::Expand;
                    }
                }
                Phase::Expand => {
                    radius += expand_rate;
                    if radius >= max_radius {
                        phase = Phase::Complete;
                    }
                }
                Phase::Complete => {}
            }

            terminal.tick();

            let spot_coords: Vec<Coord> = spotlight_ids
                .iter()
                .filter_map(|&id| {
                    terminal
                        .get_characters()
                        .iter()
                        .find(|c| c.character_id == id)
                        .map(|c| c.motion.current_coord)
                })
                .collect();
            illuminate(&mut terminal, &spot_coords, radius, &color_map);
            frames_out.push(terminal.render_frame());

            if phase == Phase::Complete {
                // Final resolution: everything fully lit in the final gradient.
                for character in terminal.get_characters_mut() {
                    if let Some(&(bright, _)) = color_map.get(&character.character_id) {
                        character.animation.set_appearance(
                            character.input_symbol,
                            Some(ColorPair::fg_only(bright)),
                        );
                    }
                }
                frames_out.push(terminal.render_frame());
                break;
            }
        }

        frames_out
    }
}
