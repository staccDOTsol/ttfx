use super::Effect;
use crate::engine::Canvas;
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

const CIPHERTEXT_COLORS: [Color; 3] = [
    Color::new(0x00, 0x80, 0x00),
    Color::new(0x00, 0xcb, 0x00),
    Color::new(0x00, 0xff, 0x00),
];

const ENCRYPTED_SYMBOLS: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '!', '@', '#',
    '$', '%', '^', '&', '*', '(', ')', '+', '-', '=', '?', '/', '\\',
    '|', '[', ']', '{', '}', '<', '>',
];

const TYPING_SPEED: usize = 1;
const DECRYPTIONS_PER_FRAME: usize = 3;
const DECRYPT_DURATION: usize = 26;

#[derive(Clone, Debug)]
struct InputCharacter {
    coord: Coord,
    symbol: String,
}

#[derive(Clone, Copy, Debug, Default)]
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
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
        let mut lines: Vec<&str> = normalized.split('\n').collect();

        if lines.last().is_some_and(|line| line.is_empty()) {
            lines.pop();
        }

        if lines.is_empty() {
            return Vec::new();
        }

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let mut characters = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                characters.push(InputCharacter {
                    coord: Coord::new(column as i32, row as i32),
                    symbol: symbol.to_string(),
                });
            }
        }

        if characters.is_empty() {
            return Vec::new();
        }

        let seed = input_seed(normalized.as_bytes());
        let mut decrypt_order: Vec<usize> = (0..characters.len()).collect();
        shuffle(&mut decrypt_order, seed ^ 0xd3c4_7a91_b825_f06d);

        let mut decrypt_rank = vec![0usize; characters.len()];
        for (rank, character_index) in decrypt_order.into_iter().enumerate() {
            decrypt_rank[character_index] = rank;
        }

        let typing_frames =
            characters.len().div_ceil(TYPING_SPEED.max(1));
        let final_gradient = Gradient::new(
            [
                Color::new(0x00, 0xd1, 0xff),
                Color::new(0xff, 0xff, 0xff),
            ],
            12,
        )
        .colors();

        let last_start = typing_frames
            + (characters.len().saturating_sub(1) / DECRYPTIONS_PER_FRAME);
        let last_frame = last_start + DECRYPT_DURATION;

        let mut canvas = Canvas::new(width, height);
        let mut frames = Vec::with_capacity(last_frame + 1);

        for frame_index in 0..=last_frame {
            canvas.clear();

            let visible_count = if frame_index < typing_frames {
                ((frame_index + 1) * TYPING_SPEED).min(characters.len())
            } else {
                characters.len()
            };

            for (character_index, character) in
                characters.iter().enumerate().take(visible_count)
            {
                let decrypt_start = typing_frames
                    + decrypt_rank[character_index] / DECRYPTIONS_PER_FRAME;

                let (symbol, color) = if frame_index < decrypt_start {
                    ciphertext_visual(
                        seed,
                        character_index,
                        frame_index,
                        frame_index,
                    )
                } else {
                    let elapsed = frame_index - decrypt_start;

                    if elapsed >= DECRYPT_DURATION {
                        (
                            character.symbol.clone(),
                            gradient_color(
                                &final_gradient,
                                character.coord,
                                width,
                                height,
                            ),
                        )
                    } else {
                        let symbol_step = slowed_symbol_step(elapsed);
                        ciphertext_visual(
                            seed ^ 0xa734_6fe2_1b90_c85d,
                            character_index,
                            frame_index,
                            symbol_step,
                        )
                    }
                };

                let style = Style::with_colors(ColorPair::new(
                    Some(color),
                    None,
                ));
                canvas.set(character.coord, symbol, style);
            }

            frames.push(canvas.render());
        }

        frames
    }
}

fn slowed_symbol_step(elapsed: usize) -> usize {
    match elapsed {
        0..=9 => elapsed,
        10..=13 => 10,
        14..=18 => 11,
        19..=24 => 12,
        _ => 13,
    }
}

fn ciphertext_visual(
    seed: u64,
    character_index: usize,
    frame_index: usize,
    symbol_step: usize,
) -> (String, Color) {
    let symbol_value = mixed_value(
        seed,
        character_index as u64,
        symbol_step as u64,
    );
    let color_value = mixed_value(
        seed ^ 0x9e37_79b9_7f4a_7c15,
        character_index as u64,
        frame_index as u64,
    );

    let symbol =
        ENCRYPTED_SYMBOLS[symbol_value as usize % ENCRYPTED_SYMBOLS.len()];
    let color =
        CIPHERTEXT_COLORS[color_value as usize % CIPHERTEXT_COLORS.len()];

    (symbol.to_string(), color)
}

fn gradient_color(
    colors: &[Color],
    coord: Coord,
    width: usize,
    height: usize,
) -> Color {
    if colors.is_empty() {
        return Color::new(0xff, 0xff, 0xff);
    }

    let maximum_distance =
        width.saturating_sub(1) + height.saturating_sub(1);
    if maximum_distance == 0 {
        return colors[0];
    }

    let distance = coord.column.max(0) as usize + coord.row.max(0) as usize;
    let index = distance
        .saturating_mul(colors.len().saturating_sub(1))
        .div_ceil(maximum_distance)
        .min(colors.len() - 1);

    colors[index]
}

fn input_seed(input: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    for byte in input {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    mix(hash ^ input.len() as u64)
}

fn shuffle(values: &mut [usize], seed: u64) {
    let mut state = seed | 1;

    for index in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;

        let other = state as usize % (index + 1);
        values.swap(index, other);
    }
}

fn mixed_value(seed: u64, first: u64, second: u64) -> u64 {
    mix(
        seed
            ^ first.wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ second.wrapping_mul(0xbf58_476d_1ce4_e5b9),
    )
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
