//! Waves effect: waves of block symbols sweep across the text column by
//! column, after which each character fades to its final gradient color.
//!
//! Port of terminaltexteffects/effects/effect_waves.py.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use super::Effect;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Characters overlaid with waves that travel across the terminal, leaving
/// behind the character colored with the final gradient.
pub struct Waves {
    /// Symbols used to build the wave (mirrors the Python default).
    wave_symbols: Vec<char>,
    /// Number of waves that pass over each character.
    wave_count: usize,
    /// Duration (ticks) of each wave frame.
    wave_length: u32,
    /// Gradient stops for the wave itself.
    wave_gradient_stops: Vec<Color>,
    /// Interpolation steps between wave gradient stops.
    wave_gradient_steps: usize,
    /// Gradient stops for the final character colors.
    final_gradient_stops: Vec<Color>,
    /// Interpolation steps between final gradient stops.
    final_gradient_steps: usize,
    /// Safety cap on the number of rendered frames.
    max_frames: usize,
}

impl Waves {
    pub fn new() -> Self {
        let hex = |s: &str| Color::from_hex(s).expect("valid hex color");
        Self {
            wave_symbols: vec![
                '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█', '▇', '▆', '▅', '▄', '▃', '▂', '▁',
            ],
            wave_count: 7,
            wave_length: 2,
            wave_gradient_stops: vec![
                hex("f0ff65"),
                hex("ffb102"),
                hex("31a0d4"),
                hex("ffb102"),
                hex("f0ff65"),
            ],
            wave_gradient_steps: 6,
            final_gradient_stops: vec![hex("ffb102"), hex("31a0d4"), hex("f0ff65")],
            final_gradient_steps: 12,
            max_frames: 5000,
        }
    }
}

impl Default for Waves {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Waves {
    fn name(&self) -> &str {
        "waves"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;

        let wave_gradient = Gradient::new(&self.wave_gradient_stops, self.wave_gradient_steps);
        let final_gradient = Gradient::new(&self.final_gradient_stops, self.final_gradient_steps);
        let wave_last_color = wave_gradient
            .spectrum
            .last()
            .copied()
            .unwrap_or(Color::new(255, 255, 255));

        // Map each character to a final color using a diagonal gradient
        // direction across the canvas (Python default: DIAGONAL).
        let mut final_color_map: HashMap<u32, Color> = HashMap::new();
        let denom = (((width - 1) + (height - 1)).max(1)) as f64;
        for character in terminal.get_characters() {
            let t = ((character.input_coord.column - 1) + (character.input_coord.row - 1)) as f64
                / denom;
            let color = final_gradient
                .get_color_at_fraction(t)
                .unwrap_or(wave_last_color);
            final_color_map.insert(character.character_id, color);
        }

        // Build the "wave" and "final" scenes for every character.
        let symbol_count = self.wave_symbols.len();
        for character in terminal.get_characters_mut() {
            let input_symbol = character.input_symbol;
            let final_color = final_color_map
                .get(&character.character_id)
                .copied()
                .unwrap_or(wave_last_color);

            // Wave scene: the wave symbols with the wave gradient applied
            // across them, repeated wave_count times.
            {
                let wave_scn = character.animation.new_scene("wave", false);
                for _ in 0..self.wave_count {
                    for (i, &symbol) in self.wave_symbols.iter().enumerate() {
                        let frac = if symbol_count > 1 {
                            i as f64 / (symbol_count - 1) as f64
                        } else {
                            0.0
                        };
                        let color = wave_gradient
                            .get_color_at_fraction(frac)
                            .unwrap_or(wave_last_color);
                        wave_scn.add_frame(
                            symbol,
                            self.wave_length,
                            Some(ColorPair::fg_only(color)),
                        );
                    }
                }
            }

            // Final scene: fade from the wave's last color to the
            // character's final gradient color.
            {
                let fade = Gradient::new(&[wave_last_color, final_color], self.final_gradient_steps);
                let final_scn = character.animation.new_scene("final", false);
                for &color in &fade.spectrum {
                    final_scn.add_frame(input_symbol, 10, Some(ColorPair::fg_only(color)));
                }
            }
        }

        // Group characters into columns, left to right.
        let mut columns_map: BTreeMap<i32, Vec<u32>> = BTreeMap::new();
        for character in terminal.get_characters() {
            columns_map
                .entry(character.input_coord.column)
                .or_default()
                .push(character.character_id);
        }
        let mut pending_columns: VecDeque<Vec<u32>> = columns_map.into_values().collect();

        // Phase per character: 0 = pending, 1 = wave running, 2 = final.
        let mut phase: HashMap<u32, u8> = HashMap::new();

        let mut frames: Vec<String> = Vec::new();
        while (!pending_columns.is_empty() || terminal.is_active())
            && frames.len() < self.max_frames
        {
            // Activate the next column's wave scenes.
            if let Some(column) = pending_columns.pop_front() {
                let ids: HashSet<u32> = column.iter().copied().collect();
                for character in terminal.get_characters_mut() {
                    if ids.contains(&character.character_id) {
                        character.is_visible = true;
                        character.animation.activate_scene("wave");
                        phase.insert(character.character_id, 1);
                    }
                }
            }

            terminal.tick();

            // Emulate the Python SCENE_COMPLETE -> ACTIVATE_SCENE event:
            // when a character's wave scene finishes, start its final fade.
            for character in terminal.get_characters_mut() {
                if phase.get(&character.character_id) == Some(&1)
                    && character.animation.active_scene_id.is_none()
                {
                    character.animation.activate_scene("final");
                    phase.insert(character.character_id, 2);
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}
