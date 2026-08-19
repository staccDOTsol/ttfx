use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Errorcorrect;

impl Errorcorrect {
    pub fn new() -> Self {
        Errorcorrect
    }
}

impl Effect for Errorcorrect {
    fn name(&self) -> &str {
        "errorcorrect"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Work with a non-empty default so render-gating always has content.
        let effective_input = if input.trim().is_empty() {
            "ErrorCorrect"
        } else {
            input
        };

        let lines: Vec<&str> = effective_input.split('\n').collect();
        let height = lines.len().max(1) as u16;
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1) as u16;

        let mut terminal = Terminal::new(width, height);

        // Character metadata for the correction animation.
        struct CharSpec {
            id: u32,
            target: Coord,
            original: char,
            corrupt_until: usize,
            transition_len: usize,
            wrong_color: Color,
            final_color: Color,
            gradient: Gradient,
        }

        let mut specs: Vec<CharSpec> = Vec::new();
        let mut next_id: u32 = 0;

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let id = next_id;
                next_id += 1;

                let target = Coord::new(col as i32, row as i32);

                // Seed is fixed so runs are deterministic.
                let seed: u64 = 0x2f6e2b1_u64;
                let corrupt_until = hash_3(seed, 0, id as u64, 7) as usize % 24;
                let wrong_color = color_from(hash_3(seed, 0, id as u64, 9));
                let final_color = Color::new(0, 255, 170);

                let gradient =
                    Gradient::new(vec![(0.0, wrong_color), (1.0, final_color)]);

                let character = EffectCharacter::new(id, target, ch);
                terminal.add_character(character);

                specs.push(CharSpec {
                    id,
                    target,
                    original: ch,
                    corrupt_until,
                    transition_len: 5,
                    wrong_color,
                    final_color,
                    gradient,
                });
            }
        }

        let wrong_symbols: Vec<char> =
            "!@#$%^&*()_+-=[]{};:,.<>?/~`0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .chars()
                .collect();
        let total_frames = 32;
        let mut frames = Vec::with_capacity(total_frames);

        for frame in 0..total_frames {
            for spec in &specs {
                if let Some(character) = terminal.get_character_mut(spec.id) {
                    let h = hash_3(0x2f6e2b1_u64, frame, spec.id as u64, 13);
                    let jitter_x = ((h % 3) as i32) - 1;
                    let jitter_y = (((h >> 8) % 3) as i32) - 1;

                    if frame < spec.corrupt_until {
                        // Corrupted state: wrong symbol, wrong color, slight coordinate jitter.
                        let symbol_index = (h as usize) % wrong_symbols.len();
                        character.symbol = wrong_symbols[symbol_index];
                        character.position = Coord::new(
                            spec.target.x + jitter_x,
                            spec.target.y + jitter_y,
                        );
                        character.color_pair = ColorPair::new(spec.wrong_color, Color::BLACK);
                        character.bold = false;
                        character.dim = true;
                    } else {
                        let transition_end = spec.corrupt_until + spec.transition_len;
                        if frame < transition_end {
                            // Transition: interpolate from wrong color to final color and
                            // switch to the correct symbol part-way through.
                            let t =
                                (frame - spec.corrupt_until) as f64 / spec.transition_len as f64;
                            let color = spec.gradient.color_at(t);

                            character.symbol = if t >= 0.5 {
                                spec.original
                            } else {
                                let symbol_index = (h as usize) % wrong_symbols.len();
                                wrong_symbols[symbol_index]
                            };
                            character.position = spec.target;
                            character.color_pair = ColorPair::new(color, Color::BLACK);
                            character.bold = true;
                            character.dim = false;
                        } else {
                            // Corrected state.
                            character.symbol = spec.original;
                            character.position = spec.target;
                            character.color_pair =
                                ColorPair::new(spec.final_color, Color::BLACK);
                            character.bold = true;
                            character.dim = false;
                        }
                    }
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}

fn hash_3(seed: u64, frame: usize, id: u64, salt: u64) -> u64 {
    let mut x = seed
        ^ id.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (frame as u64).wrapping_mul(0xD1B5_4A32_D192_ED03)
        ^ salt.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = x.wrapping_add(0x94D0_49BB_1331_11EB);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x
}

fn color_from(value: u64) -> Color {
    const PALETTE: [Color; 4] = [
        Color {
            r: 255,
            g: 60,
            b: 60,
        },
        Color {
            r: 255,
            g: 160,
            b: 0,
        },
        Color {
            r: 230,
            g: 230,
            b: 0,
        },
        Color {
            r: 255,
            g: 0,
            b: 127,
        },
    ];
    PALETTE[(value as usize) % PALETTE.len()]
}
