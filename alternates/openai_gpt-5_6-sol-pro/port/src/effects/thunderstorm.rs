use super::Effect;

use crate::engine::Canvas;
use crate::utils::{Color, Coord, Gradient, Style};

pub struct Thunderstorm;

impl Thunderstorm {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Thunderstorm {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Thunderstorm {
    fn name(&self) -> &str {
        "thunderstorm"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let text = TextLayout::new(input);
        let mut frames = Vec::new();
        let mut rng = StormRng::new(hash_input(input));
        let mut rain = Vec::<RainDrop>::new();

        let storm_gradient = Gradient::new(
            [
                Color::new(8, 12, 24),
                Color::new(18, 29, 52),
                Color::new(31, 48, 76),
            ],
            12,
        )
        .colors();

        for step in 0..12 {
            let color = storm_gradient[step];
            frames.push(render_frame(
                &text,
                color,
                &rain,
                &[],
                None,
                step as u64,
            ));
        }

        const STRIKES: [usize; 3] = [14, 39, 65];

        for storm_frame in 0..84 {
            update_rain(
                &mut rain,
                &mut rng,
                text.width,
                text.height,
                storm_frame,
            );

            let mut bolt = Vec::new();
            let mut flash = None;

            for (strike_index, strike_start) in STRIKES.iter().enumerate() {
                if storm_frame >= *strike_start && storm_frame < strike_start + 6 {
                    let age = storm_frame - strike_start;
                    bolt = lightning_bolt(
                        text.width,
                        text.height,
                        hash_input(input)
                            ^ ((*strike_start as u64 + 1) * 0x9e37_79b9)
                            ^ strike_index as u64,
                    );

                    flash = Some(match age {
                        0 => Flash::Faint,
                        1 | 2 => Flash::Bright,
                        3 => Flash::Cool,
                        _ => Flash::Afterglow,
                    });
                    break;
                }
            }

            let rumble = ((storm_frame as f64 * 0.31).sin() * 3.0).round() as i16;
            let base = Color::new(
                (20_i16 + rumble).clamp(8, 30) as u8,
                (31_i16 + rumble).clamp(14, 42) as u8,
                (51_i16 + rumble * 2).clamp(24, 68) as u8,
            );

            frames.push(render_frame(
                &text,
                base,
                &rain,
                &bolt,
                flash,
                storm_frame as u64 + 12,
            ));
        }

        let final_gradient = Gradient::new(
            [
                Color::new(34, 52, 79),
                Color::new(89, 132, 174),
                Color::new(194, 222, 238),
                Color::new(235, 246, 250),
            ],
            18,
        )
        .colors();

        rain.clear();
        for (step, color) in final_gradient.into_iter().enumerate() {
            frames.push(render_frame(
                &text,
                color,
                &rain,
                &[],
                None,
                96 + step as u64,
            ));
        }

        frames
    }
}

#[derive(Clone)]
struct TextCell {
    coord: Coord,
    symbol: String,
}

struct TextLayout {
    width: usize,
    height: usize,
    cells: Vec<TextCell>,
}

impl TextLayout {
    fn new(input: &str) -> Self {
        let normalized = input.strip_suffix('\n').unwrap_or(input);
        let normalized = normalized.strip_suffix('\r').unwrap_or(normalized);
        let lines = normalized.split('\n').collect::<Vec<_>>();

        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);
        let mut cells = Vec::new();

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                cells.push(TextCell {
                    coord: Coord::new(column as i32, row as i32),
                    symbol: symbol.to_string(),
                });
            }
        }

        if cells.is_empty() {
            cells.push(TextCell {
                coord: Coord::new(0, 0),
                symbol: " ".to_owned(),
            });
        }

        Self {
            width,
            height,
            cells,
        }
    }
}

#[derive(Clone, Copy)]
struct RainDrop {
    column: i32,
    row: i32,
    speed: i32,
    length: i32,
}

#[derive(Clone, Copy)]
enum Flash {
    Faint,
    Bright,
    Cool,
    Afterglow,
}

fn update_rain(
    rain: &mut Vec<RainDrop>,
    rng: &mut StormRng,
    width: usize,
    height: usize,
    frame: usize,
) {
    for drop in rain.iter_mut() {
        drop.row += drop.speed;
    }

    rain.retain(|drop| drop.row - drop.length < height as i32);

    let density = ((width + 3) / 4).max(1);
    let spawn_count = density + usize::from(frame % 3 == 0);

    for _ in 0..spawn_count {
        if rng.next_u32() % 100 < 72 {
            rain.push(RainDrop {
                column: rng.range(width as u32) as i32,
                row: -(rng.range(4) as i32) - 1,
                speed: 1 + rng.range(2) as i32,
                length: 1 + rng.range(3) as i32,
            });
        }
    }
}

fn lightning_bolt(width: usize, height: usize, seed: u64) -> Vec<Coord> {
    let mut rng = StormRng::new(seed);
    let mut bolt = Vec::new();
    let mut column = rng.range(width as u32) as i32;

    for row in 0..height as i32 {
        bolt.push(Coord::new(column, row));

        if row + 1 < height as i32 {
            let movement = rng.range(3) as i32 - 1;
            column = (column + movement).clamp(0, width as i32 - 1);
        }

        if row > 0 && row + 1 < height as i32 && rng.next_u32() % 100 < 28 {
            let direction = if rng.next_u32() & 1 == 0 { -1 } else { 1 };
            let branch_column =
                (column + direction).clamp(0, width as i32 - 1);
            bolt.push(Coord::new(branch_column, row));
        }
    }

    bolt
}

fn render_frame(
    text: &TextLayout,
    text_color: Color,
    rain: &[RainDrop],
    bolt: &[Coord],
    flash: Option<Flash>,
    frame: u64,
) -> String {
    let mut canvas = Canvas::new(text.width, text.height);

    let flash_background = match flash {
        Some(Flash::Bright) => Some(Color::new(20, 29, 48)),
        Some(Flash::Faint) => Some(Color::new(8, 13, 25)),
        _ => None,
    };

    for cell in &text.cells {
        let near_bolt = bolt
            .iter()
            .any(|coord| coord.manhattan_distance_to(cell.coord) <= 2);

        let foreground = if near_bolt {
            match flash {
                Some(Flash::Bright) => Color::new(218, 239, 255),
                Some(Flash::Cool) => Color::new(121, 184, 235),
                Some(Flash::Afterglow) => Color::new(75, 121, 166),
                _ => text_color.lerp(Color::new(135, 179, 214), 0.45),
            }
        } else {
            text_color
        };

        canvas.set(
            cell.coord,
            cell.symbol.clone(),
            Style {
                foreground: Some(foreground),
                background: flash_background,
                bold: near_bolt && matches!(flash, Some(Flash::Bright)),
                ..Style::default()
            },
        );
    }

    for drop in rain {
        for offset in 0..drop.length {
            let coord = Coord::new(drop.column, drop.row - offset);
            if !canvas.contains(coord) {
                continue;
            }

            let leading = offset == 0;
            let shimmer = ((frame + drop.column as u64) & 3) == 0;
            let color = if leading {
                Color::new(99, 145, 187)
            } else if shimmer {
                Color::new(56, 91, 126)
            } else {
                Color::new(38, 66, 98)
            };

            canvas.set(
                coord,
                if leading { "│" } else { "·" },
                Style {
                    foreground: Some(color),
                    ..Style::default()
                },
            );
        }
    }

    for (index, coord) in bolt.iter().enumerate() {
        if !canvas.contains(*coord) {
            continue;
        }

        let symbol = if index == 0 {
            "╷"
        } else if index + 1 == bolt.len() {
            "╵"
        } else if index % 3 == 0 {
            "╲"
        } else if index % 3 == 1 {
            "│"
        } else {
            "╱"
        };

        let color = match flash {
            Some(Flash::Bright) => Color::new(245, 252, 255),
            Some(Flash::Cool) => Color::new(160, 211, 247),
            Some(Flash::Afterglow) => Color::new(82, 137, 184),
            _ => Color::new(132, 180, 220),
        };

        canvas.set(
            *coord,
            symbol,
            Style {
                foreground: Some(color),
                background: flash_background,
                bold: matches!(flash, Some(Flash::Bright)),
                ..Style::default()
            },
        );
    }

    canvas.render()
}

fn hash_input(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash ^ 0xa076_1d64_78bd_642f
}

struct StormRng {
    state: u64,
}

impl StormRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x6a09_e667_f3bc_c909
            } else {
                seed
            },
        }
    }

    fn next_u32(&mut self) -> u32 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        (value >> 16) as u32
    }

    fn range(&mut self, upper: u32) -> u32 {
        if upper == 0 {
            0
        } else {
            self.next_u32() % upper
        }
    }
}
