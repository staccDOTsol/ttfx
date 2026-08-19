//! Sweep effect: a dim noise sweep passes over the canvas revealing the
//! characters as blocky static, then a second sweep passes back across and
//! resolves each character to its final gradient color.
//!
//! Port of terminaltexteffects/effects/effect_sweep.py.

use std::collections::{BTreeMap, VecDeque};

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Block symbols cycled while a character is unresolved noise.
const SWEEP_SYMBOLS: [char; 4] = ['█', '▓', '▒', '░'];

/// Hard cap on emitted frames, mirroring the bounded run loop.
const MAX_FRAMES: usize = 2000;

/// Fixed PRNG seed so output is reproducible.
const SWEEP_SEED: u64 = 0x5EED_0000_51EE_D001;

/// Small deterministic PRNG so noise symbol choice is reproducible
/// without pulling in an external rand crate.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed | 1)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    fn choice<T: Copy>(&mut self, items: &[T]) -> T {
        items[(self.next_u32() as usize) % items.len()]
    }
}

pub struct Sweep;

impl Sweep {
    pub fn new() -> Self {
        Sweep
    }
}

/// Activate a scene on the character with the given id.
fn activate_scene(terminal: &mut Terminal, id: u32, scene_id: &str) {
    if let Some(character) = terminal
        .get_characters_mut()
        .iter_mut()
        .find(|c| c.character_id == id)
    {
        character.animation.activate_scene(scene_id);
    }
}

impl Effect for Sweep {
    fn name(&self) -> &str {
        "sweep"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let mut rng = Lcg::new(SWEEP_SEED);

        // Defaults mirroring the Python effect config: dim gray noise for the
        // first sweep, and the standard purple -> cyan -> white final gradient.
        let noise_color = Color::from_hex("404040").expect("valid hex");
        let final_stops = [
            Color::from_hex("8A008A").expect("valid hex"),
            Color::from_hex("00D1FF").expect("valid hex"),
            Color::from_hex("FFFFFF").expect("valid hex"),
        ];
        let final_gradient = Gradient::new(&final_stops, 12);

        let width = terminal.canvas.width;
        let height = terminal.canvas.height;

        // Snapshot character info before taking mutable borrows.
        let char_info: Vec<(u32, char, i32, i32)> = terminal
            .get_characters()
            .iter()
            .map(|c| (c.character_id, c.input_symbol, c.input_coord.column, c.input_coord.row))
            .collect();

        // Build per-character scenes: a looping "noise" scene of random sweep
        // symbols, and a non-looping "sweep" scene fading from the noise color
        // to the character's final gradient color, ending on the input symbol.
        for &(id, symbol, column, row) in &char_info {
            // Diagonal gradient mapping across the canvas.
            let denom = ((height - 1) + (width - 1)).max(1) as f64;
            let frac = ((row - 1) + (column - 1)) as f64 / denom;
            let final_color = final_gradient
                .get_color_at_fraction(frac)
                .unwrap_or(final_stops[final_stops.len() - 1]);

            // Pre-pick the noise symbols for this character.
            let noise_syms: Vec<char> = (0..4).map(|_| rng.choice(&SWEEP_SYMBOLS)).collect();

            let fade = Gradient::new(&[noise_color, final_color], 3);

            if let Some(character) = terminal
                .get_characters_mut()
                .iter_mut()
                .find(|c| c.character_id == id)
            {
                // Looping static while waiting for the resolving sweep.
                let noise_scn = character.animation.new_scene("noise", true);
                for &sym in &noise_syms {
                    noise_scn.add_frame(sym, 2, Some(ColorPair::fg_only(noise_color)));
                }

                // Resolve: descend through the block symbols while the color
                // brightens toward the final gradient color, then settle on
                // the input symbol.
                let sweep_scn = character.animation.new_scene("sweep", false);
                for (i, &sym) in SWEEP_SYMBOLS.iter().enumerate() {
                    let color = fade
                        .spectrum
                        .get(i.min(fade.spectrum.len().saturating_sub(1)))
                        .copied()
                        .unwrap_or(final_color);
                    sweep_scn.add_frame(sym, 2, Some(ColorPair::fg_only(color)));
                }
                sweep_scn.add_frame(symbol, 1, Some(ColorPair::fg_only(final_color)));
            }
        }

        // Group character ids by column for the two sweep passes.
        let mut columns: BTreeMap<i32, Vec<u32>> = BTreeMap::new();
        for &(id, _, column, _) in &char_info {
            columns.entry(column).or_default().push(id);
        }

        // First sweep travels right-to-left revealing noise; the second sweep
        // travels back left-to-right resolving characters.
        let mut first_sweep: VecDeque<Vec<u32>> = columns.values().rev().cloned().collect();
        let mut second_sweep: VecDeque<Vec<u32>> = columns.values().cloned().collect();

        let mut frames: Vec<String> = Vec::new();
        frames.push(terminal.render_frame());

        let mut emitted = 0usize;
        loop {
            if let Some(group) = first_sweep.pop_front() {
                for id in group {
                    terminal.set_character_visibility(id, true);
                    activate_scene(&mut terminal, id, "noise");
                }
            } else if let Some(group) = second_sweep.pop_front() {
                for id in group {
                    activate_scene(&mut terminal, id, "sweep");
                }
            }

            terminal.tick();
            frames.push(terminal.render_frame());
            emitted += 1;

            if emitted >= MAX_FRAMES {
                break;
            }
            if first_sweep.is_empty() && second_sweep.is_empty() && !terminal.is_active() {
                break;
            }
        }

        frames
    }
}
