
use super::Effect;
use crate::engine::{
    CharacterId, CharacterVisual, EffectCharacter, Frame, Path, Scene,
    Terminal, Waypoint,
};
use crate::utils::{
    Color, ColorPair, Coord, Gradient, Style,
};

pub struct Rain;

impl Rain {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Rain {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Rain {
    fn name(&self) -> &str {
        "rain"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut lines: Vec<&str> = input.split('\n').collect();
        if input.ends_with('\n') {
            lines.pop();
        }

        for line in &mut lines {
            *line = line.strip_suffix('\r').unwrap_or(line);
        }

        if lines.is_empty() {
            lines.push("");
        }

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let rain_colors = [
            Color::new(0x00, 0x31, 0x5c),
            Color::new(0x00, 0x4c, 0x6d),
            Color::new(0x00, 0x7f, 0x7b),
            Color::new(0x00, 0xb1, 0x79),
            Color::new(0x81, 0xd1, 0x52),
            Color::new(0xf9, 0xf8, 0x71),
        ];

        let final_colors = Gradient::new(
            [
                Color::new(0x8a, 0x00, 0x8a),
                Color::new(0x00, 0xd1, 0xff),
                Color::new(0xff, 0xff, 0xff),
            ],
            height.max(12),
        )
        .colors();

        let mut rng = RainRng::new(hash_input(input));
        let mut terminal = Terminal::new(width, height);
        let mut character_count = 0usize;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                let target = Coord::new(column as i32, row as i32);
                let distance_above = rng.range_usize(1, height.max(1)) as i32;
                let start = Coord::new(column as i32, -distance_above);

                let rain_color =
                    rain_colors[rng.range_usize(0, rain_colors.len() - 1)];
                let gradient_row = height.saturating_sub(row + 1);
                let gradient_index = if height <= 1 {
                    0
                } else {
                    gradient_row * (final_colors.len() - 1) / (height - 1)
                };
                let final_color = final_colors[gradient_index];

                let mut character = EffectCharacter::new(
                    CharacterId(character_count as u32),
                    symbol.to_string(),
                    start,
                );
                character.visible = false;
                character.style = Style::with_colors(ColorPair::new(
                    Some(rain_color),
                    None,
                ));

                let speed = 0.1 + rng.next_f64() * 0.1;
                let mut path = Path::new("fall", speed);
                path.add_waypoint(Waypoint::new("input", target));
                character.motion.add_path(path);

                let transition =
                    Gradient::new([rain_color, final_color], 7).colors();
                let mut scene = Scene::new("settle", false);

                for color in transition {
                    scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            symbol.to_string(),
                            Style::with_colors(ColorPair::new(
                                Some(color),
                                None,
                            )),
                        ),
                        5,
                    ));
                }

                character.animation.add_scene(scene);
                terminal.add_character(character);
                character_count += 1;
            }
        }

        if character_count == 0 {
            return Vec::new();
        }

        let mut pending: Vec<usize> = (0..character_count).collect();
        rng.shuffle(&mut pending);

        let mut pending_index = 0usize;
        let mut falling = vec![false; character_count];
        let mut delay = 0usize;
        let maximum_frames = 256usize
            .saturating_add(character_count.saturating_mul(6))
            .saturating_add(height.saturating_mul(24))
            .min(100_000);

        let mut frames = Vec::new();

        for _ in 0..maximum_frames {
            if pending_index < pending.len() {
                if delay == 0 {
                    let batch_size = rng.range_usize(1, 3);

                    for _ in 0..batch_size {
                        let Some(&character_index) =
                            pending.get(pending_index)
                        else {
                            break;
                        };
                        pending_index += 1;

                        let character =
                            &mut terminal.characters_mut()[character_index];
                        character.visible = true;
                        character
                            .motion
                            .activate("fall", character.position);
                        falling[character_index] = true;
                    }

                    delay = rng.range_usize(1, 3);
                } else {
                    delay -= 1;
                }
            }

            let frame = terminal.step_frame();

            for (index, character) in
                terminal.characters_mut().iter_mut().enumerate()
            {
                if falling[index] && !character.motion.is_active() {
                    falling[index] = false;
                    character.animation.activate("settle");
                }
            }

            if frame.contains("\x1b[") {
                frames.push(frame);
            }

            let pending_remains = pending_index < pending.len();
            let falling_remains = falling.iter().any(|is_falling| *is_falling);
            let animation_remains = terminal
                .characters()
                .iter()
                .any(|character| character.animation.is_active());

            if !pending_remains && !falling_remains && !animation_remains {
                break;
            }
        }

        frames
    }
}

fn hash_input(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;

    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    if hash == 0 {
        0x9e37_79b9_7f4a_7c15
    } else {
        hash
    }
}

struct RainRng {
    state: u64,
}

impl RainRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    fn range_usize(&mut self, minimum: usize, maximum: usize) -> usize {
        if maximum <= minimum {
            return minimum;
        }

        minimum + self.next_u64() as usize % (maximum - minimum + 1)
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let other = self.range_usize(0, index);
            values.swap(index, other);
        }
    }
}
