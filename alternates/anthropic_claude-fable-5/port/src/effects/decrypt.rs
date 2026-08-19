//! Decrypt effect: movie-style decryption. The text is first "typed" onto the
//! screen as ciphertext, then each character rapidly cycles through random
//! encrypted symbols (with occasional slow, "stuck" symbols) before being
//! discovered and resolving to the plaintext in the final gradient color.
//!
//! Port of terminaltexteffects/effects/effect_decrypt.py.

use std::collections::VecDeque;

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Small deterministic xorshift PRNG so the effect needs no external crates.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Random integer in `[lo, hi)`.
    fn gen_range(&mut self, lo: u64, hi: u64) -> u64 {
        let span = (hi.saturating_sub(lo)).max(1);
        lo + self.next_u64() % span
    }

    /// Random element of a non-empty slice.
    fn choice<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next_u64() % items.len() as u64) as usize]
    }

    /// Random integer in `0..=100` (percent roll, mirrors `random.randint(0, 100)`).
    fn percent(&mut self) -> u64 {
        self.next_u64() % 101
    }
}

/// Pool of "encrypted" symbols, mirroring the Python effect's character ranges:
/// printable ASCII plus Latin Extended and Cyrillic blocks.
fn encrypted_symbols() -> Vec<char> {
    let mut symbols = Vec::new();
    for n in 33u32..127 {
        if let Some(c) = char::from_u32(n) {
            symbols.push(c);
        }
    }
    for n in 161u32..381 {
        if let Some(c) = char::from_u32(n) {
            symbols.push(c);
        }
    }
    for n in 1024u32..1120 {
        if let Some(c) = char::from_u32(n) {
            symbols.push(c);
        }
    }
    symbols
}

pub struct Decrypt;

impl Decrypt {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Decrypt {
    fn name(&self) -> &str {
        "decrypt"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        const MAX_FRAMES: usize = 20000;
        const TYPING_SPEED: usize = 1;

        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let mut rng = Rng::new(0xDEC0_DE5E_ED01);
        let symbols = encrypted_symbols();

        // Defaults from the Python effect config.
        let ciphertext_colors = [
            Color::from_hex("008000").expect("valid hex"),
            Color::from_hex("00cb00").expect("valid hex"),
            Color::from_hex("00ff00").expect("valid hex"),
        ];
        let final_stop = Color::from_hex("eda000").expect("valid hex");
        let final_gradient = Gradient::new(&[final_stop], 12);
        let height = terminal.canvas.height;

        // Allocation order == input order, which drives the typing sequence.
        let ids: Vec<u32> = terminal
            .get_characters()
            .iter()
            .map(|c| c.character_id)
            .collect();

        // Build the typing and decrypting scenes for every character.
        for character in terminal.get_characters_mut() {
            let input_symbol = character.input_symbol;

            // Final color mapped vertically across the canvas.
            let t = if height > 1 {
                (character.input_coord.row - 1) as f64 / (height - 1) as f64
            } else {
                0.0
            };
            let final_color = final_gradient.get_color_at_fraction(t).unwrap_or(final_stop);

            // Typing scene: cursor blocks fading out, then a ciphertext symbol
            // that persists until decryption begins.
            {
                let cursor_color = *rng.choice(&ciphertext_colors);
                let symbol_color = *rng.choice(&ciphertext_colors);
                let typed_symbol = *rng.choice(&symbols);
                let scene = character.animation.new_scene("typing", false);
                for block in ['█', '▓', '▒', '░'] {
                    scene.add_frame(block, 2, Some(ColorPair::fg_only(cursor_color)));
                }
                scene.add_frame(typed_symbol, 2, Some(ColorPair::fg_only(symbol_color)));
            }

            // Decrypting scene: fast cycling, then slow cycling with occasional
            // long "stuck" symbols, then a white-to-final discovery gradient.
            {
                let color = *rng.choice(&ciphertext_colors);
                let slow_count = rng.gen_range(1, 11);
                let mut slow_frames: Vec<(char, u32)> = Vec::new();
                for _ in 0..slow_count {
                    let sym = *rng.choice(&symbols);
                    let duration = if rng.percent() <= 30 {
                        rng.gen_range(20, 45)
                    } else {
                        rng.gen_range(3, 8)
                    } as u32;
                    slow_frames.push((sym, duration));
                }
                let mut fast_frames: Vec<char> = Vec::new();
                for _ in 0..40 {
                    fast_frames.push(*rng.choice(&symbols));
                }
                let discovered = Gradient::new(&[Color::new(255, 255, 255), final_color], 10);

                let scene = character.animation.new_scene("decrypting", false);
                for sym in fast_frames {
                    scene.add_frame(sym, 2, Some(ColorPair::fg_only(color)));
                }
                for (sym, duration) in slow_frames {
                    scene.add_frame(sym, duration, Some(ColorPair::fg_only(color)));
                }
                for step_color in &discovered.spectrum {
                    scene.add_frame(input_symbol, 3, Some(ColorPair::fg_only(*step_color)));
                }
                scene.add_frame(input_symbol, 1, Some(ColorPair::fg_only(final_color)));
            }
        }

        let mut frames_out = Vec::new();
        frames_out.push(terminal.render_frame());

        // Phase 1: typing. Each tick has a 75% chance of typing the next
        // character(s), as in the Python original.
        let mut pending: VecDeque<u32> = ids.into_iter().collect();
        loop {
            if !pending.is_empty() && rng.percent() <= 75 {
                for _ in 0..TYPING_SPEED {
                    if let Some(id) = pending.pop_front() {
                        terminal.set_character_visibility(id, true);
                        if let Some(character) = terminal
                            .get_characters_mut()
                            .iter_mut()
                            .find(|c| c.character_id == id)
                        {
                            character.animation.activate_scene("typing");
                        }
                    }
                }
            }
            terminal.tick();
            frames_out.push(terminal.render_frame());
            if pending.is_empty() && !terminal.is_active() {
                break;
            }
            if frames_out.len() >= MAX_FRAMES {
                return frames_out;
            }
        }

        // Phase 2: decrypting. All characters begin cycling simultaneously.
        for character in terminal.get_characters_mut() {
            character.animation.activate_scene("decrypting");
        }
        while terminal.is_active() && frames_out.len() < MAX_FRAMES {
            terminal.tick();
            frames_out.push(terminal.render_frame());
        }

        frames_out
    }
}
