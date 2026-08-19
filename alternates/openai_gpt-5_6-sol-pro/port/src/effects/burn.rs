
use super::Effect;
use crate::engine::{
    Canvas, CharacterId, CharacterVisual, EffectCharacter, Frame, Scene,
};
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

const STARTING_COLOR: Color = Color::new(0x83, 0x73, 0x73);
const BURN_COLORS: [Color; 5] = [
    Color::new(0xff, 0xff, 0xff),
    Color::new(0xff, 0xf7, 0x5d),
    Color::new(0xfe, 0x65, 0x0d),
    Color::new(0x8a, 0x00, 0x3c),
    Color::new(0x51, 0x01, 0x00),
];
const FINAL_COLORS: [Color; 2] = [
    Color::new(0x00, 0xc3, 0xff),
    Color::new(0xff, 0xff, 0x1c),
];
const BURN_SYMBOLS: [&str; 10] =
    ["▂", "▃", "▄", "▅", "▆", "▇", "█", "▓", "▒", "░"];

pub struct Burn;

impl Burn {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Burn {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Burn {
    fn name(&self) -> &str {
        "burn"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input
            .lines()
            .map(|line| line.trim_end_matches('\r'))
            .collect();

        if lines.is_empty() {
            let mut canvas = Canvas::new(1, 1);
            canvas.fill(" ", colored_style(STARTING_COLOR));
            return vec![canvas.render()];
        }

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);
        let final_colors =
            Gradient::new(FINAL_COLORS, height).colors();
        let burn_colors =
            Gradient::new(BURN_COLORS, BURN_SYMBOLS.len()).colors();

        let mut characters = Vec::new();
        let mut rows = vec![Vec::new(); height];
        let mut next_id = 0_u32;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let mut character = EffectCharacter::new(
                    CharacterId(next_id),
                    symbol.to_string(),
                    Coord::new(column as i32, row as i32),
                );
                next_id = next_id.saturating_add(1);

                character.style = colored_style(STARTING_COLOR);

                let final_color = final_colors
                    .get(row)
                    .copied()
                    .unwrap_or(FINAL_COLORS[0]);
                let mut burn_scene = Scene::new("burn", false);

                for (index, burn_symbol) in BURN_SYMBOLS.iter().enumerate() {
                    let color = burn_colors
                        .get(index)
                        .copied()
                        .unwrap_or(BURN_COLORS[BURN_COLORS.len() - 1]);

                    burn_scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            *burn_symbol,
                            colored_style(color),
                        ),
                        2,
                    ));
                }

                burn_scene.add_frame(Frame::new(
                    CharacterVisual::new(
                        symbol.to_string(),
                        colored_style(final_color),
                    ),
                    1,
                ));
                character.animation.add_scene(burn_scene);

                let index = characters.len();
                characters.push(character);
                rows[row].push(index);
            }
        }

        if characters.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.fill(" ", colored_style(STARTING_COLOR));
            return vec![canvas.render()];
        }

        let mut random_state = seed_from_input(input);
        for row in &mut rows {
            shuffle(row, &mut random_state);
        }

        // Burning proceeds from the bottom row toward the top.
        rows.reverse();

        let mut pending_rows = rows;
        let mut frames = Vec::new();
        let mut canvas = Canvas::new(width, height);

        loop {
            while pending_rows.first().is_some_and(Vec::is_empty) {
                pending_rows.remove(0);
            }

            if let Some(row) = pending_rows.first_mut() {
                let ignition_count =
                    1 + (next_random(&mut random_state) as usize % 3);
                let mut ignite = Vec::new();

                for _ in 0..ignition_count {
                    if let Some(index) = row.pop() {
                        ignite.push(index);
                    } else {
                        break;
                    }
                }

                for index in ignite {
                    characters[index].animation.activate("burn");
                }
            }

            for character in &mut characters {
                character.step();
            }

            canvas.fill(" ", colored_style(STARTING_COLOR));
            for character in &characters {
                canvas.draw_character(character);
            }
            frames.push(canvas.render());

            let pending = pending_rows.iter().any(|row| !row.is_empty());
            let active = characters.iter().any(EffectCharacter::is_active);

            if !pending && !active {
                break;
            }
        }

        frames
    }
}

fn colored_style(color: Color) -> Style {
    Style::with_colors(ColorPair::new(Some(color), None))
}

fn seed_from_input(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    if hash == 0 {
        0x9e37_79b9_7f4a_7c15
    } else {
        hash
    }
}

fn next_random(state: &mut u64) -> u64 {
    let mut value = *state;
    value ^= value << 13;
    value ^= value >> 7;
    value ^= value << 17;
    *state = value;
    value
}

fn shuffle(values: &mut [usize], state: &mut u64) {
    for index in (1..values.len()).rev() {
        let other = next_random(state) as usize % (index + 1);
        values.swap(index, other);
    }
}
