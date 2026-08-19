use super::Effect;

use crate::engine::{
    CharacterId, CharacterVisual, EffectCharacter, Frame, Path, Scene, Terminal,
    Waypoint,
};
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

pub struct Swarm;

impl Swarm {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Swarm {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Swarm {
    fn name(&self) -> &str {
        "swarm"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let normalized = input.trim_end_matches(&['\r', '\n'][..]);
        let lines: Vec<Vec<char>> = if normalized.is_empty() {
            vec![vec![' ']]
        } else {
            normalized
                .lines()
                .map(|line| line.trim_end_matches('\r').chars().collect())
                .collect()
        };

        let width = lines
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let mut terminal = Terminal::new(width, height);
        let mut rng = SwarmRng::new(hash_input(input));

        let base_color = Color::new(0x31, 0xa0, 0xd4);
        let flash_color = Color::new(0xf2, 0xe7, 0xde);
        let final_colors = Gradient::new(
            [
                Color::new(0x31, 0xa0, 0xd4),
                Color::new(0x8a, 0x00, 0x8a),
            ],
            width.saturating_add(height).max(2),
        )
        .colors();
        let pulse_colors =
            Gradient::new([base_color, flash_color, base_color], 9).colors();

        let targets: Vec<(char, Coord)> = lines
            .iter()
            .enumerate()
            .flat_map(|(row, line)| {
                line.iter().enumerate().map(move |(column, symbol)| {
                    (
                        *symbol,
                        Coord::new(column as i32, row as i32),
                    )
                })
            })
            .collect();

        const SWARM_SIZE: usize = 10;
        const SWARM_AREA_COUNT: usize = 3;
        const SWARM_COORDINATION: f64 = 0.8;
        const MOVEMENT_SPEED: f64 = 0.8;

        let mut next_id = 0_u32;

        for group in targets.chunks(SWARM_SIZE) {
            let start_anchor = random_edge_coord(&mut rng, width, height);
            let swarm_areas: Vec<Coord> = (0..SWARM_AREA_COUNT)
                .map(|_| random_canvas_coord(&mut rng, width, height))
                .collect();

            for (symbol, target) in group {
                let start = coordinated_coord(
                    &mut rng,
                    start_anchor,
                    width,
                    height,
                    2,
                );

                let diagonal = target.column.max(0) as usize
                    + target.row.max(0) as usize;
                let final_color = final_colors
                    [diagonal.min(final_colors.len().saturating_sub(1))];

                let mut character = EffectCharacter::new(
                    CharacterId(next_id),
                    symbol.to_string(),
                    start,
                );
                next_id = next_id.wrapping_add(1);

                character.style = Style::with_colors(ColorPair::new(
                    Some(base_color),
                    None,
                ));

                let mut path = Path::new("swarm", MOVEMENT_SPEED);
                for (area_index, area) in swarm_areas.iter().enumerate() {
                    let waypoint = if rng.next_f64() < SWARM_COORDINATION {
                        coordinated_coord(
                            &mut rng,
                            *area,
                            width,
                            height,
                            3,
                        )
                    } else {
                        random_canvas_coord(&mut rng, width, height)
                    };

                    path.add_waypoint(Waypoint::new(
                        format!("area-{area_index}"),
                        waypoint,
                    ));
                }
                path.add_waypoint(Waypoint::new("input", *target));
                character.motion.add_path(path);
                character.motion.activate("swarm", start);

                let mut scene = Scene::new("swarm-flash", false);
                let delay = rng.range_usize(1, 5) as u32;
                scene.add_frame(Frame::new(
                    CharacterVisual::new(
                        symbol.to_string(),
                        Style::with_colors(ColorPair::new(
                            Some(base_color),
                            None,
                        )),
                    ),
                    delay,
                ));

                for color in &pulse_colors {
                    scene.add_frame(Frame::new(
                        CharacterVisual::new(
                            symbol.to_string(),
                            Style::with_colors(ColorPair::new(
                                Some(*color),
                                None,
                            )),
                        ),
                        2,
                    ));
                }

                scene.add_frame(Frame::new(
                    CharacterVisual::new(
                        symbol.to_string(),
                        Style::with_colors(ColorPair::new(
                            Some(final_color),
                            None,
                        )),
                    ),
                    8,
                ));

                character.animation.add_scene(scene);
                character.animation.activate("swarm-flash");
                terminal.add_character(character);
            }
        }

        let max_frames = width
            .saturating_add(height)
            .saturating_mul(12)
            .clamp(120, 4000);
        terminal.run(max_frames)
    }
}

fn coordinated_coord(
    rng: &mut SwarmRng,
    anchor: Coord,
    width: usize,
    height: usize,
    radius: i32,
) -> Coord {
    let column = anchor.column + rng.range_i32(-radius, radius + 1);
    let row = anchor.row + rng.range_i32(-radius, radius + 1);

    Coord::new(
        column.clamp(0, width.saturating_sub(1) as i32),
        row.clamp(0, height.saturating_sub(1) as i32),
    )
}

fn random_canvas_coord(
    rng: &mut SwarmRng,
    width: usize,
    height: usize,
) -> Coord {
    Coord::new(
        rng.range_usize(0, width.max(1)) as i32,
        rng.range_usize(0, height.max(1)) as i32,
    )
}

fn random_edge_coord(
    rng: &mut SwarmRng,
    width: usize,
    height: usize,
) -> Coord {
    let right = width.saturating_sub(1) as i32;
    let bottom = height.saturating_sub(1) as i32;

    match rng.range_usize(0, 4) {
        0 => Coord::new(rng.range_usize(0, width.max(1)) as i32, 0),
        1 => Coord::new(right, rng.range_usize(0, height.max(1)) as i32),
        2 => Coord::new(
            rng.range_usize(0, width.max(1)) as i32,
            bottom,
        ),
        _ => Coord::new(0, rng.range_usize(0, height.max(1)) as i32),
    }
}

fn hash_input(input: &str) -> u64 {
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

struct SwarmRng {
    state: u64,
}

impl SwarmRng {
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
        let value = self.next_u64() >> 11;
        value as f64 / ((1_u64 << 53) as f64)
    }

    fn range_usize(&mut self, start: usize, end: usize) -> usize {
        if end <= start {
            return start;
        }

        start + self.next_u64() as usize % (end - start)
    }

    fn range_i32(&mut self, start: i32, end: i32) -> i32 {
        if end <= start {
            return start;
        }

        start + (self.next_u64() % (end - start) as u64) as i32
    }
}
