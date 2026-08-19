
use super::Effect;
use crate::engine::{
    Canvas, CharacterId, CharacterVisual, EffectCharacter, Frame, Path, Scene,
    Waypoint,
};
use crate::utils::{Color, Coord, Gradient, Style};

const SMOKE_DELAY: usize = 5;
const SMOKE_SCENE: &str = "smoke";
const FINAL_SCENE: &str = "final";

pub struct Smoke;

impl Smoke {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Smoke {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Smoke {
    fn name(&self) -> &str {
        "smoke"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines = input
            .lines()
            .map(|line| line.trim_end_matches('\r').to_owned())
            .collect::<Vec<_>>();

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let mut canvas = Canvas::new(width, height);
        let mut rng = SmokeRng::new(seed_from_input(input));

        let smoke_colors = Gradient::new(
            [
                Color::new(58, 58, 58),
                Color::new(98, 98, 98),
                Color::new(154, 154, 154),
                Color::new(210, 210, 210),
            ],
            12,
        )
        .colors();

        let final_colors = Gradient::new(
            [
                Color::new(138, 0, 138),
                Color::new(0, 209, 255),
                Color::new(255, 255, 255),
            ],
            height,
        )
        .colors();

        let mut characters = Vec::new();
        let mut next_id = 0_u32;

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                if symbol.is_whitespace() {
                    continue;
                }

                let target = Coord::new(column as i32, row as i32);
                let horizontal_drift = rng.range_i32(-2, 2);
                let starting_column =
                    (target.column + horizontal_drift).clamp(0, width as i32 - 1);
                let starting_position =
                    Coord::new(starting_column, height as i32 - 1);

                let smoke_offset = rng.range_usize(0, smoke_colors.len() - 1);
                let speed = 0.35 + rng.range_usize(0, 25) as f64 / 100.0;
                let final_color = final_colors
                    .get(row)
                    .copied()
                    .unwrap_or(Color::new(255, 255, 255));

                let mut character = EffectCharacter::new(
                    CharacterId(next_id),
                    symbol.to_string(),
                    starting_position,
                );
                next_id += 1;
                character.visible = false;

                let mut smoke_scene = Scene::new(SMOKE_SCENE, true);
                let smoke_symbols = ["▂", "▃", "▄", "▅", "▆", "▇", "█", "▓", "▒", "░"];

                for index in 0..smoke_symbols.len() {
                    let color =
                        smoke_colors[(index + smoke_offset) % smoke_colors.len()];
                    smoke_scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            smoke_symbols[index],
                            foreground_style(color),
                        ),
                        2,
                    ));
                }

                let transition_start =
                    smoke_colors[(smoke_offset + smoke_colors.len() - 1)
                        % smoke_colors.len()];
                let transition_colors =
                    Gradient::new([transition_start, final_color], 7).colors();

                let mut final_scene = Scene::new(FINAL_SCENE, false);
                let transition_symbols = ["▓", "▒", "░"];

                for (index, transition_symbol) in
                    transition_symbols.iter().enumerate()
                {
                    let color = transition_colors
                        .get(index)
                        .copied()
                        .unwrap_or(transition_start);
                    final_scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            *transition_symbol,
                            foreground_style(color),
                        ),
                        2,
                    ));
                }

                for color in transition_colors
                    .iter()
                    .copied()
                    .skip(transition_symbols.len())
                {
                    final_scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            symbol.to_string(),
                            foreground_style(color),
                        ),
                        2,
                    ));
                }

                final_scene.add_frame(Frame::new(
                    CharacterVisual::new(
                        symbol.to_string(),
                        foreground_style(final_color),
                    ),
                    1,
                ));

                character.animation.add_scene(smoke_scene);
                character.animation.add_scene(final_scene);

                let mut path = Path::new(SMOKE_SCENE, speed);
                path.add_waypoint(Waypoint::new("input", target));
                character.motion.add_path(path);

                characters.push(SmokeCharacter {
                    character,
                    phase: SmokePhase::Pending,
                });
            }
        }

        if characters.is_empty() {
            canvas.fill(
                " ",
                foreground_style(Color::new(154, 154, 154)),
            );
            return vec![canvas.render()];
        }

        let mut launch_order = (0..characters.len()).collect::<Vec<_>>();
        rng.shuffle(&mut launch_order);

        let mut launch_index = 0;
        let mut delay_counter = SMOKE_DELAY;
        let mut frames = Vec::new();

        loop {
            if launch_index < launch_order.len()
                && delay_counter >= SMOKE_DELAY
            {
                let index = launch_order[launch_index];
                launch_index += 1;
                delay_counter = 0;

                let smoke_character = &mut characters[index];
                smoke_character.character.visible = true;
                smoke_character.character.motion.activate(
                    SMOKE_SCENE,
                    smoke_character.character.position,
                );
                smoke_character.character.animation.activate(SMOKE_SCENE);

                if let Some(visual) = smoke_character
                    .character
                    .animation
                    .current_visual()
                    .cloned()
                {
                    smoke_character.character.set_appearance(visual);
                }

                smoke_character.phase = SmokePhase::Rising;
            } else {
                delay_counter += 1;
            }

            for smoke_character in &mut characters {
                match smoke_character.phase {
                    SmokePhase::Pending | SmokePhase::Settled => {}
                    SmokePhase::Rising => {
                        smoke_character.character.step();

                        if !smoke_character.character.motion.is_active() {
                            smoke_character
                                .character
                                .animation
                                .activate(FINAL_SCENE);

                            if let Some(visual) = smoke_character
                                .character
                                .animation
                                .current_visual()
                                .cloned()
                            {
                                smoke_character
                                    .character
                                    .set_appearance(visual);
                            }

                            smoke_character.phase = SmokePhase::Coalescing;
                        }
                    }
                    SmokePhase::Coalescing => {
                        smoke_character.character.step();

                        if !smoke_character.character.animation.is_active() {
                            smoke_character.phase = SmokePhase::Settled;
                        }
                    }
                }
            }

            canvas.clear();
            for smoke_character in &characters {
                if smoke_character.phase != SmokePhase::Pending {
                    canvas.draw_character(&smoke_character.character);
                }
            }
            frames.push(canvas.render());

            if launch_index >= launch_order.len()
                && characters
                    .iter()
                    .all(|character| character.phase == SmokePhase::Settled)
            {
                break;
            }
        }

        frames
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SmokePhase {
    Pending,
    Rising,
    Coalescing,
    Settled,
}

struct SmokeCharacter {
    character: EffectCharacter,
    phase: SmokePhase,
}

fn foreground_style(color: Color) -> Style {
    Style {
        foreground: Some(color),
        ..Style::default()
    }
}

fn seed_from_input(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

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

struct SmokeRng {
    state: u64,
}

impl SmokeRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn range_usize(&mut self, minimum: usize, maximum: usize) -> usize {
        if minimum >= maximum {
            return minimum;
        }

        minimum + self.next_u64() as usize % (maximum - minimum + 1)
    }

    fn range_i32(&mut self, minimum: i32, maximum: i32) -> i32 {
        if minimum >= maximum {
            return minimum;
        }

        minimum
            + (self.next_u64() % (maximum - minimum + 1) as u64) as i32
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let replacement = self.range_usize(0, index);
            values.swap(index, replacement);
        }
    }
}
