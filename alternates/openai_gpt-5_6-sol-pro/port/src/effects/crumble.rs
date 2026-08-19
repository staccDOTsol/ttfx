use super::Effect;

use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::graphics::{Color, ColorPair, Gradient, Style};
use crate::utils::Coord;

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
        let parsed = ParsedText::new(input);
        let mut order = (0..parsed.characters.len()).collect::<Vec<_>>();
        deterministic_shuffle(&mut order, input);

        let mut rank = vec![0usize; parsed.characters.len()];
        for (position, character_index) in order.into_iter().enumerate() {
            rank[character_index] = position;
        }

        let character_count = parsed.characters.len().max(1);
        let batch_size = character_count.div_ceil(30).max(1);
        let stagger_count = character_count.saturating_sub(1) / batch_size;
        let bottom = parsed.height.saturating_sub(1) as i32;
        let top = 0;
        let center_column = parsed.width.saturating_sub(1) as f64 / 2.0;

        const WEAKEN_DURATION: usize = 8;
        let fall_duration = (parsed.height * 2 + 10).clamp(12, 42);
        let fall_phase_duration =
            stagger_count * 2 + WEAKEN_DURATION + fall_duration + 3;

        let vacuum_duration = (parsed.height * 2 + 12).clamp(16, 46);
        let vacuum_stagger = character_count.saturating_sub(1) / batch_size;
        let vacuum_phase_duration = vacuum_stagger + vacuum_duration + 3;

        let reform_duration = (parsed.height * 2 + 12).clamp(16, 46);
        let reform_phase_duration = vacuum_stagger + reform_duration + 3;

        let mut frames = Vec::with_capacity(
            fall_phase_duration
                + vacuum_phase_duration
                + reform_phase_duration
                + 1,
        );

        for tick in 0..fall_phase_duration {
            frames.push(parsed.render(|index, character| {
                let start = (rank[index] / batch_size) * 2;

                if tick < start {
                    RenderedCharacter::original(character)
                } else if tick < start + WEAKEN_DURATION {
                    let progress =
                        (tick - start) as f64 / WEAKEN_DURATION as f64;
                    let symbols = [
                        character.symbol.as_str(),
                        character.symbol.as_str(),
                        "▓",
                        "▒",
                        "░",
                    ];
                    let symbol_index = ((progress * symbols.len() as f64)
                        .floor() as usize)
                        .min(symbols.len() - 1);

                    RenderedCharacter {
                        coord: character.coord,
                        symbol: symbols[symbol_index].to_owned(),
                        color: character
                            .final_color
                            .lerp(Color::new(115, 105, 98), progress),
                    }
                } else {
                    let elapsed = tick - start - WEAKEN_DURATION;
                    let progress =
                        (elapsed as f64 / fall_duration as f64).clamp(0.0, 1.0);
                    let eased = easing::out_bounce(progress);
                    let row = interpolate(
                        character.coord.row as f64,
                        bottom as f64,
                        eased,
                    );

                    RenderedCharacter {
                        coord: Coord::new(character.coord.column, row),
                        symbol: if progress < 0.4 {
                            "▓".to_owned()
                        } else if progress < 0.75 {
                            "▒".to_owned()
                        } else {
                            "░".to_owned()
                        },
                        color: character
                            .final_color
                            .lerp(Color::new(105, 93, 85), progress * 0.8),
                    }
                }
            }));
        }

        for tick in 0..vacuum_phase_duration {
            frames.push(parsed.render(|index, character| {
                let start = rank[index] / batch_size;
                let elapsed = tick.saturating_sub(start);
                let progress = if tick < start {
                    0.0
                } else {
                    (elapsed as f64 / vacuum_duration as f64).clamp(0.0, 1.0)
                };
                let eased = easing::out_quint(progress);
                let row = interpolate(bottom as f64, top as f64, eased);

                // Pull the debris toward the center as it is vacuumed upward,
                // approximating the curved top path used by the original.
                let arc = (std::f64::consts::PI * progress).sin() * 0.42;
                let column = interpolate(
                    character.coord.column as f64,
                    center_column,
                    arc,
                );

                RenderedCharacter {
                    coord: Coord::new(column, row),
                    symbol: if progress < 0.55 {
                        "░".to_owned()
                    } else if progress < 0.85 {
                        "▒".to_owned()
                    } else {
                        "▓".to_owned()
                    },
                    color: Color::new(105, 93, 85)
                        .lerp(character.final_color, progress * 0.45),
                }
            }));
        }

        for tick in 0..reform_phase_duration {
            frames.push(parsed.render(|index, character| {
                let reverse_rank = character_count - 1 - rank[index];
                let start = reverse_rank / batch_size;
                let elapsed = tick.saturating_sub(start);
                let progress = if tick < start {
                    0.0
                } else {
                    (elapsed as f64 / reform_duration as f64).clamp(0.0, 1.0)
                };
                let eased = easing::out_bounce(progress);
                let row =
                    interpolate(top as f64, character.coord.row as f64, eased);

                RenderedCharacter {
                    coord: Coord::new(character.coord.column, row),
                    symbol: if progress < 0.2 {
                        "░".to_owned()
                    } else if progress < 0.4 {
                        "▒".to_owned()
                    } else if progress < 0.6 {
                        "▓".to_owned()
                    } else {
                        character.symbol.clone()
                    },
                    color: Color::new(125, 113, 104)
                        .lerp(character.final_color, progress),
                }
            }));
        }

        frames.push(parsed.render(|_, character| {
            RenderedCharacter::original(character)
        }));
        frames
    }
}

#[derive(Clone)]
struct InputCharacter {
    symbol: String,
    coord: Coord,
    final_color: Color,
}

struct RenderedCharacter {
    coord: Coord,
    symbol: String,
    color: Color,
}

impl RenderedCharacter {
    fn original(character: &InputCharacter) -> Self {
        Self {
            coord: character.coord,
            symbol: character.symbol.clone(),
            color: character.final_color,
        }
    }
}

struct ParsedText {
    width: usize,
    height: usize,
    characters: Vec<InputCharacter>,
    fallback_color: Color,
}

impl ParsedText {
    fn new(input: &str) -> Self {
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        let mut lines = normalized.split('\n').collect::<Vec<_>>();

        while lines.len() > 1 && lines.last().is_some_and(|line| line.is_empty())
        {
            lines.pop();
        }

        if lines.is_empty() {
            lines.push("");
        }

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let colors = Gradient::new(
            [
                Color::new(92, 225, 255),
                Color::new(125, 94, 255),
                Color::new(255, 140, 0),
            ],
            height,
        )
        .colors();
        let fallback_color = colors
            .first()
            .copied()
            .unwrap_or(Color::new(92, 225, 255));

        let mut characters = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            let color = colors.get(row).copied().unwrap_or(fallback_color);
            for (column, symbol) in line.chars().enumerate() {
                characters.push(InputCharacter {
                    symbol: symbol.to_string(),
                    coord: Coord::new(column as i32, row as i32),
                    final_color: color,
                });
            }
        }

        Self {
            width,
            height,
            characters,
            fallback_color,
        }
    }

    fn render(
        &self,
        mut transform: impl FnMut(usize, &InputCharacter) -> RenderedCharacter,
    ) -> String {
        let mut canvas = Canvas::new(self.width, self.height);

        if self.characters.is_empty() {
            canvas.set(
                Coord::new(0, 0),
                " ",
                Style::with_colors(ColorPair::new(
                    Some(self.fallback_color),
                    None,
                )),
            );
        } else {
            for (index, character) in self.characters.iter().enumerate() {
                let rendered = transform(index, character);
                canvas.set(
                    rendered.coord,
                    rendered.symbol,
                    Style::with_colors(ColorPair::new(
                        Some(rendered.color),
                        None,
                    )),
                );
            }
        }

        canvas.render()
    }
}

fn interpolate(start: f64, end: f64, progress: f64) -> i32 {
    (start + (end - start) * progress.clamp(0.0, 1.0)).round() as i32
}

fn deterministic_shuffle(values: &mut [usize], input: &str) {
    let mut state = 0xcbf2_9ce4_8422_2325_u64;

    for byte in input.bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x0000_0100_0000_01b3);
    }

    for index in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;

        let swap_index = (state as usize) % (index + 1);
        values.swap(index, swap_index);
    }
}
