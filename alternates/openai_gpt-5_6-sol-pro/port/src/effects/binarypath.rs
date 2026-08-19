
use super::Effect;
use crate::engine::{
    Canvas, CharacterId, CharacterVisual, EffectCharacter, Frame, Path, Scene,
    Waypoint,
};
use crate::utils::{Color, Coord, Gradient, Style};

pub struct Binarypath;

impl Binarypath {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Binarypath {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Binarypath {
    fn name(&self) -> &str {
        "binarypath"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect()).collect();
        let width = lines
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let final_colors = Gradient::new(
            [
                Color::new(0x00, 0xd5, 0x00),
                Color::new(0x00, 0xff, 0x88),
                Color::new(0x00, 0xbb, 0xff),
            ],
            width.saturating_add(height).saturating_sub(1).max(1),
        )
        .colors();

        let mut particles = Vec::new();

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.iter().copied().enumerate() {
                let target = Coord::new(column as i32, row as i32);
                let id = particles.len() as u32;
                let seed = mix_seed(
                    id as u64
                        ^ ((column as u64) << 21)
                        ^ ((row as u64) << 42)
                        ^ symbol as u64,
                );
                let start = starting_coord(seed, target, width, height);
                let final_color = final_colors[(column + row) % final_colors.len()];

                let mut character =
                    EffectCharacter::new(CharacterId(id), binary_digit(symbol, id), start);
                character.visible = false;
                character.style = binary_style(Color::new(0x00, 0xff, 0x55));

                let mut path = Path::new("binary_path", 1.0);
                add_binary_waypoints(&mut path, seed, start, target, width, height);
                character.motion.add_path(path);

                let mut transition = Scene::new("resolve", false);
                let active_color = Color::new(0xe0, 0xff, 0xe0);

                for step in 0..8 {
                    let progress = step as f64 / 7.0;
                    let color = active_color.lerp(final_color, progress);
                    let bit = binary_digit(symbol, id.wrapping_add(step as u32));

                    transition.add_frame(Frame::new(
                        CharacterVisual::new(bit, binary_style(color)),
                        1,
                    ));
                }

                transition.add_frame(Frame::new(
                    CharacterVisual::new(
                        symbol.to_string(),
                        binary_style(final_color),
                    ),
                    5,
                ));
                character.animation.add_scene(transition);

                particles.push(BinaryParticle {
                    character,
                    target,
                    original: symbol,
                    delay: particle_delay(id, particles.len()),
                    started: false,
                    arrived: false,
                });
            }
        }

        if particles.is_empty() {
            let mut canvas = Canvas::new(width, height);
            canvas.set(
                Coord::new(0, 0),
                " ",
                binary_style(Color::new(0x00, 0xd5, 0x00)),
            );
            return vec![canvas.render()];
        }

        let mut output = Vec::new();
        let maximum_frames = width
            .saturating_add(height)
            .saturating_mul(4)
            .saturating_add(128);

        for frame_index in 0..maximum_frames {
            let mut canvas = Canvas::new(width, height);

            for particle in &mut particles {
                if !particle.started && frame_index >= particle.delay {
                    particle.started = true;
                    particle.character.visible = true;
                    particle.character.motion.activate(
                        "binary_path",
                        particle.character.position,
                    );
                }

                if !particle.started {
                    continue;
                }

                if !particle.arrived {
                    let bit_index = frame_index
                        .wrapping_add(particle.character.id.0 as usize);
                    particle.character.symbol =
                        binary_digit(particle.original, bit_index as u32);
                    particle.character.style =
                        binary_style(binary_palette(bit_index));

                    particle.character.step();

                    if !particle.character.motion.is_active() {
                        particle.character.position = particle.target;
                        particle.arrived = true;
                        particle.character.animation.activate("resolve");

                        if let Some(visual) =
                            particle.character.animation.current_visual().cloned()
                        {
                            particle.character.set_appearance(visual);
                        }
                    }
                } else if particle.character.animation.is_active() {
                    particle.character.step();
                }

                canvas.draw_character(&particle.character);
            }

            output.push(canvas.render());

            if particles.iter().all(|particle| {
                particle.started
                    && particle.arrived
                    && !particle.character.animation.is_active()
                    && !particle.character.motion.is_active()
            }) {
                break;
            }
        }

        output
    }
}

struct BinaryParticle {
    character: EffectCharacter,
    target: Coord,
    original: char,
    delay: usize,
    started: bool,
    arrived: bool,
}

fn binary_style(color: Color) -> Style {
    Style {
        foreground: Some(color),
        bold: true,
        ..Style::default()
    }
}

fn binary_palette(index: usize) -> Color {
    const COLORS: [Color; 5] = [
        Color::new(0x04, 0x4e, 0x29),
        Color::new(0x15, 0x7e, 0x38),
        Color::new(0x45, 0xbf, 0x55),
        Color::new(0x95, 0xed, 0x87),
        Color::new(0xe0, 0xff, 0xe0),
    ];

    COLORS[index % COLORS.len()]
}

fn binary_digit(symbol: char, bit_index: u32) -> String {
    let value = symbol as u32;
    let shift = 7 - (bit_index % 8);
    if (value >> shift) & 1 == 0 {
        "0".to_owned()
    } else {
        "1".to_owned()
    }
}

fn particle_delay(id: u32, index: usize) -> usize {
    if index == 0 {
        0
    } else {
        ((id as usize).wrapping_mul(17).wrapping_add(index * 7)) % 36
    }
}

fn starting_coord(
    seed: u64,
    target: Coord,
    width: usize,
    height: usize,
) -> Coord {
    let right = width.saturating_sub(1) as i32;
    let bottom = height.saturating_sub(1) as i32;
    let random_column = ((seed >> 8) as usize % width) as i32;
    let random_row = ((seed >> 24) as usize % height) as i32;

    match seed & 3 {
        0 => Coord::new(0, random_row),
        1 => Coord::new(right, random_row),
        2 => Coord::new(random_column, 0),
        _ => Coord::new(random_column, bottom),
    }
    .or_target_if_equal(target, right, bottom)
}

trait DistinctStart {
    fn or_target_if_equal(
        self,
        target: Coord,
        right: i32,
        bottom: i32,
    ) -> Coord;
}

impl DistinctStart for Coord {
    fn or_target_if_equal(
        self,
        target: Coord,
        right: i32,
        bottom: i32,
    ) -> Coord {
        if self != target {
            return self;
        }

        if target.column != right {
            Coord::new(right, target.row)
        } else if target.column != 0 {
            Coord::new(0, target.row)
        } else if target.row != bottom {
            Coord::new(target.column, bottom)
        } else {
            Coord::new(target.column, 0)
        }
    }
}

fn add_binary_waypoints(
    path: &mut Path,
    seed: u64,
    start: Coord,
    target: Coord,
    width: usize,
    height: usize,
) {
    let bend_column = ((seed >> 16) as usize % width) as i32;
    let bend_row = ((seed >> 32) as usize % height) as i32;

    if seed & 4 == 0 {
        path.add_waypoint(Waypoint::new(
            "horizontal_entry",
            Coord::new(bend_column, start.row),
        ));
        path.add_waypoint(Waypoint::new(
            "vertical_channel",
            Coord::new(bend_column, bend_row),
        ));
        path.add_waypoint(Waypoint::new(
            "target_row",
            Coord::new(target.column, bend_row),
        ));
    } else {
        path.add_waypoint(Waypoint::new(
            "vertical_entry",
            Coord::new(start.column, bend_row),
        ));
        path.add_waypoint(Waypoint::new(
            "horizontal_channel",
            Coord::new(bend_column, bend_row),
        ));
        path.add_waypoint(Waypoint::new(
            "target_column",
            Coord::new(bend_column, target.row),
        ));
    }

    path.add_waypoint(Waypoint::new("target", target));
}

fn mix_seed(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
