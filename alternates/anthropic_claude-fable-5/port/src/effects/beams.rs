//! Beams effect: light beams sweep across rows and down columns, illuminating
//! characters as they pass. Illuminated characters fade to a dim version of
//! their final color, then a final wipe brightens the text into the final
//! gradient. Port of terminaltexteffects/effects/effect_beams.py.

use std::collections::BTreeMap;

use super::Effect;
use crate::engine::animation::Scene;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

// ---------------------------------------------------------------------------
// Small deterministic PRNG (xorshift64) — no external rand dependency.
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(if seed == 0 { 0x9e3779b97f4a7c15 } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

fn shuffle<T>(v: &mut [T], rng: &mut Rng) {
    if v.len() < 2 {
        return;
    }
    for i in (1..v.len()).rev() {
        let j = rng.below(i + 1);
        v.swap(i, j);
    }
}

// ---------------------------------------------------------------------------
// Beam groups: one per row and one per column of the input text.
// ---------------------------------------------------------------------------

struct Group {
    char_ids: Vec<u32>,
    scene_id: &'static str,
    next_index: usize,
    counter: f64,
    speed: f64,
}

impl Group {
    fn complete(&self) -> bool {
        self.next_index >= self.char_ids.len()
    }

    /// Advance the beam along its group, lighting up characters it passes.
    fn tick(&mut self, terminal: &mut Terminal) {
        self.counter += self.speed;
        while self.counter >= 1.0 && !self.complete() {
            self.counter -= 1.0;
            let id = self.char_ids[self.next_index];
            self.next_index += 1;
            if let Some(character) = terminal
                .get_characters_mut()
                .iter_mut()
                .find(|c| c.character_id == id)
            {
                character.is_visible = true;
                character.animation.activate_scene(self.scene_id);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Mirror of Animation.apply_gradient_to_symbols: spread `symbols` evenly
/// across the gradient spectrum, one frame per spectrum color.
fn add_gradient_symbols(scene: &mut Scene, gradient: &Gradient, symbols: &[char], duration: u32) {
    let n = gradient.spectrum.len().max(1);
    for (i, color) in gradient.spectrum.iter().enumerate() {
        let idx = (i * symbols.len()) / n;
        let symbol = symbols[idx.min(symbols.len().saturating_sub(1))];
        scene.add_frame(symbol, duration, Some(ColorPair::fg_only(*color)));
    }
}

/// Mirror of Animation.adjust_color_brightness with a 0..=1 factor.
fn adjust_brightness(color: Color, factor: f64) -> Color {
    let scale = |v: u8| -> u8 { (v as f64 * factor).round().clamp(0.0, 255.0) as u8 };
    Color::new(scale(color.r), scale(color.g), scale(color.b))
}

// ---------------------------------------------------------------------------
// Effect
// ---------------------------------------------------------------------------

/// The beams effect.
pub struct Beams {
    beam_delay: u32,
    beam_gradient_frames: u32,
    final_gradient_frames: u32,
    final_wipe_speed: usize,
}

impl Beams {
    pub fn new() -> Self {
        Self {
            beam_delay: 10,
            beam_gradient_frames: 2,
            final_gradient_frames: 5,
            final_wipe_speed: 1,
        }
    }
}

impl Default for Beams {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Beams {
    fn name(&self) -> &str {
        "beams"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let mut rng = Rng::new(0x5eed_beef_cafe_f00d);

        // Config defaults from the Python original.
        let beam_row_symbols: [char; 3] = ['▂', '▁', '_'];
        let beam_column_symbols: [char; 4] = ['▌', '▍', '▎', '▏'];
        let white = Color::from_hex("ffffff").expect("valid hex");
        let cyan = Color::from_hex("00D1FF").expect("valid hex");
        let purple = Color::from_hex("8A008A").expect("valid hex");

        let beam_gradient = Gradient::new(&[white, cyan, purple], 4);
        let final_gradient = Gradient::new(&[purple, cyan, white], 12);
        let beam_end_color = *beam_gradient.spectrum.last().unwrap_or(&purple);

        let height = terminal.canvas.height;

        // -- build: scenes for every character ------------------------------
        for character in terminal.get_characters_mut() {
            // Final color: vertical gradient across the canvas (bottom -> top).
            let t = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient.get_color_at_fraction(t).unwrap_or(white);
            let faded_color = adjust_brightness(final_color, 0.3);
            let symbol = character.input_symbol;

            let fade_gradient = Gradient::new(&[beam_end_color, faded_color], 10);

            // Row beam scene: beam symbols wiping through the beam gradient,
            // then the input symbol fading down to the dim final color.
            {
                let scene = character.animation.new_scene("beam_row", false);
                add_gradient_symbols(
                    scene,
                    &beam_gradient,
                    &beam_row_symbols,
                    self.beam_gradient_frames,
                );
                for color in &fade_gradient.spectrum {
                    scene.add_frame(symbol, 5, Some(ColorPair::fg_only(*color)));
                }
            }
            // Column beam scene: same, with the column symbols.
            {
                let scene = character.animation.new_scene("beam_column", false);
                add_gradient_symbols(
                    scene,
                    &beam_gradient,
                    &beam_column_symbols,
                    self.beam_gradient_frames,
                );
                for color in &fade_gradient.spectrum {
                    scene.add_frame(symbol, 5, Some(ColorPair::fg_only(*color)));
                }
            }
            // Brighten scene: dim final color up to the full final color.
            {
                let brighten_gradient = Gradient::new(&[faded_color, final_color], 10);
                let scene = character.animation.new_scene("brighten", false);
                for color in &brighten_gradient.spectrum {
                    scene.add_frame(
                        symbol,
                        self.final_gradient_frames,
                        Some(ColorPair::fg_only(*color)),
                    );
                }
            }
        }

        // -- build: row and column groups -----------------------------------
        let mut rows: BTreeMap<i32, Vec<(i32, u32)>> = BTreeMap::new();
        let mut columns: BTreeMap<i32, Vec<(i32, u32)>> = BTreeMap::new();
        for character in terminal.get_characters() {
            rows.entry(character.input_coord.row)
                .or_default()
                .push((character.input_coord.column, character.character_id));
            columns
                .entry(character.input_coord.column)
                .or_default()
                .push((character.input_coord.row, character.character_id));
        }

        let mut pending: Vec<Group> = Vec::new();
        for members in rows.values() {
            let mut members = members.clone();
            members.sort_by_key(|(column, _)| *column); // beam sweeps left -> right
            pending.push(Group {
                char_ids: members.into_iter().map(|(_, id)| id).collect(),
                scene_id: "beam_row",
                next_index: 0,
                counter: 0.0,
                speed: rng.uniform(1.0, 4.0), // row speed range (10, 40) / 10
            });
        }
        for members in columns.values() {
            let mut members = members.clone();
            members.sort_by_key(|(row, _)| -*row); // beam falls top -> bottom
            pending.push(Group {
                char_ids: members.into_iter().map(|(_, id)| id).collect(),
                scene_id: "beam_column",
                next_index: 0,
                counter: 0.0,
                speed: rng.uniform(0.6, 1.0), // column speed range (6, 10) / 10
            });
        }
        shuffle(&mut pending, &mut rng);

        // Final wipe: rows of character ids, wiped top -> bottom.
        // Keys ascend, so pop() yields the topmost remaining row.
        let mut wipe_rows: Vec<Vec<u32>> = rows
            .values()
            .map(|members| members.iter().map(|(_, id)| *id).collect())
            .collect();

        // -- run -------------------------------------------------------------
        const PHASE_BEAMS: u8 = 0;
        const PHASE_WAIT: u8 = 1;
        const PHASE_WIPE: u8 = 2;

        let mut phase = PHASE_BEAMS;
        let mut active: Vec<Group> = Vec::new();
        let mut delay: u32 = 0;
        let max_frames: usize = 4000;

        let mut frames_out = Vec::new();
        frames_out.push(terminal.render_frame());

        while frames_out.len() < max_frames {
            match phase {
                PHASE_BEAMS => {
                    if delay == 0 {
                        if let Some(group) = pending.pop() {
                            active.push(group);
                        }
                        delay = self.beam_delay;
                    } else {
                        delay -= 1;
                    }
                    for group in active.iter_mut() {
                        group.tick(&mut terminal);
                    }
                    active.retain(|g| !g.complete());
                    if pending.is_empty() && active.is_empty() {
                        phase = PHASE_WAIT;
                    }
                }
                PHASE_WAIT => {
                    // Let the fade scenes play out before the final wipe.
                    if terminal
                        .get_characters()
                        .iter()
                        .all(|c| c.animation.active_scene_is_complete())
                    {
                        phase = PHASE_WIPE;
                    }
                }
                PHASE_WIPE => {
                    for _ in 0..self.final_wipe_speed.max(1) {
                        if let Some(row_ids) = wipe_rows.pop() {
                            for id in row_ids {
                                if let Some(character) = terminal
                                    .get_characters_mut()
                                    .iter_mut()
                                    .find(|c| c.character_id == id)
                                {
                                    character.is_visible = true;
                                    character.animation.activate_scene("brighten");
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            terminal.tick();
            frames_out.push(terminal.render_frame());

            if phase == PHASE_WIPE
                && wipe_rows.is_empty()
                && terminal
                    .get_characters()
                    .iter()
                    .all(|c| c.animation.active_scene_is_complete())
            {
                break;
            }
        }

        frames_out
    }
}
