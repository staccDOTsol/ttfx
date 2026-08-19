use std::f64::consts::PI;

use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::{Color, ColorPair, Coord, Gradient, Style};

pub struct Blackhole;

impl Blackhole {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Blackhole {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Blackhole {
    fn name(&self) -> &str {
        "blackhole"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let width = lines
            .iter()
            .map(|line| line.trim_end_matches('\r').chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = lines.len().max(1);

        let final_gradient = Gradient::new(
            [
                Color::new(0x5f, 0x1f, 0x99),
                Color::new(0x00, 0xd4, 0xff),
                Color::new(0xff, 0xff, 0xff),
                Color::new(0xff, 0xb0, 0x00),
            ],
            96,
        )
        .colors();
        let blackhole_gradient = Gradient::new(
            [
                Color::new(0x20, 0x00, 0x38),
                Color::new(0x76, 0x18, 0xb8),
                Color::new(0xff, 0x36, 0xd8),
                Color::new(0xff, 0xf0, 0x91),
            ],
            48,
        )
        .colors();
        let fade_gradient = Gradient::new(
            [
                Color::new(0x08, 0x05, 0x12),
                Color::new(0x24, 0x10, 0x40),
                Color::new(0x78, 0x2e, 0xa8),
            ],
            32,
        )
        .colors();

        let mut characters = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.trim_end_matches('\r').chars().enumerate() {
                if symbol.is_whitespace() {
                    continue;
                }

                let hash = mix(
                    column as u64
                        ^ ((row as u64) << 21)
                        ^ symbol as u32 as u64,
                );
                let color_index = ((row * width + column) * final_gradient.len())
                    / (width * height).max(1);

                characters.push(BlackholeCharacter {
                    symbol: symbol.to_string(),
                    input_coord: Coord::new(column as i32, row as i32),
                    final_color: final_gradient
                        [color_index.min(final_gradient.len() - 1)],
                    hash,
                });
            }
        }

        if characters.is_empty() {
            return vec![render_frame(
                width,
                height,
                vec![Draw {
                    coord: Coord::new(0, 0),
                    symbol: " ".to_owned(),
                    color: Color::new(0x76, 0x18, 0xb8),
                    bold: false,
                }],
            )];
        }

        let center = Coord::new(
            (width.saturating_sub(1) / 2) as i32,
            (height.saturating_sub(1) / 2) as i32,
        );
        let visual_width = width as f64;
        let visual_height = height as f64 * 2.0;
        let radius = (visual_width.min(visual_height) * 0.22)
            .max(1.5)
            .min((visual_width.max(2.0) - 1.0) / 2.0);

        let ring_count = ((characters.len() + 7) / 8)
            .max(3)
            .min(characters.len());
        let mut ranked: Vec<usize> = (0..characters.len()).collect();
        ranked.sort_by_key(|&index| characters[index].hash);
        let ring_indices = ranked[..ring_count].to_vec();

        let mut is_ring = vec![false; characters.len()];
        for &index in &ring_indices {
            is_ring[index] = true;
        }

        let remaining_indices: Vec<usize> = ranked[ring_count..].to_vec();
        let mut frames = Vec::new();

        // The input first appears as dim matter suspended around the future singularity.
        for frame in 0..12 {
            let progress = (frame + 1) as f64 / 12.0;
            let eased = easing::out_sine(progress);
            let mut draws = Vec::with_capacity(characters.len());

            for character in &characters {
                let shimmer = ((character.hash >> (frame % 16)) & 7) as usize;
                let color_index = ((eased * 20.0) as usize + shimmer)
                    .min(fade_gradient.len() - 1);
                draws.push(Draw {
                    coord: character.input_coord,
                    symbol: character.symbol.clone(),
                    color: fade_gradient[color_index],
                    bold: false,
                });
            }

            frames.push(render_frame(width, height, draws));
        }

        // A subset of the text forms the rotating event horizon.
        for frame in 0..30 {
            let progress = easing::in_out_sine((frame + 1) as f64 / 30.0);
            let mut draws = Vec::with_capacity(characters.len());

            for (index, character) in characters.iter().enumerate() {
                if !is_ring[index] {
                    draws.push(Draw {
                        coord: character.input_coord,
                        symbol: character.symbol.clone(),
                        color: character
                            .final_color
                            .lerp(Color::new(0x18, 0x08, 0x28), progress * 0.8),
                        bold: false,
                    });
                }
            }

            for (ring_position, &index) in ring_indices.iter().enumerate() {
                let angle = ring_position as f64 / ring_count as f64 * PI * 2.0
                    + progress * PI / 3.0;
                let destination = ring_coord(center, radius, angle);
                let coord = characters[index]
                    .input_coord
                    .lerp(destination, progress);
                let color_index =
                    (ring_position * 7 + frame) % blackhole_gradient.len();

                draws.push(Draw {
                    coord,
                    symbol: if progress > 0.35 {
                        "*".to_owned()
                    } else {
                        characters[index].symbol.clone()
                    },
                    color: blackhole_gradient[color_index],
                    bold: true,
                });
            }

            frames.push(render_frame(width, height, draws));
        }

        // Matter spirals into the black hole while the event horizon rotates.
        let consumption_frames =
            (38 + remaining_indices.len().saturating_mul(2)).min(110);
        for frame in 0..consumption_frames {
            let global = (frame + 1) as f64 / consumption_frames as f64;
            let mut draws = Vec::with_capacity(characters.len());

            for (order, &index) in remaining_indices.iter().enumerate() {
                let start = if remaining_indices.is_empty() {
                    0.0
                } else {
                    order as f64 / remaining_indices.len() as f64 * 0.72
                };
                let local = ((global - start) / 0.28).clamp(0.0, 1.0);

                if local >= 1.0 {
                    continue;
                }

                let character = &characters[index];
                let pull = easing::in_expo(local);
                let dx = character.input_coord.column as f64 - center.column as f64;
                let dy =
                    (character.input_coord.row as f64 - center.row as f64) * 2.0;
                let initial_radius = dx.hypot(dy);
                let initial_angle = dy.atan2(dx);
                let angle = initial_angle + pull * PI * 2.25;
                let current_radius = initial_radius * (1.0 - pull);
                let coord = Coord::new(
                    (center.column as f64 + current_radius * angle.cos()).round()
                        as i32,
                    (center.row as f64
                        + current_radius * angle.sin() / 2.0)
                        .round() as i32,
                );
                let color = character.final_color.lerp(
                    Color::new(0x72, 0x16, 0xa0),
                    (pull * 1.2).min(1.0),
                );

                draws.push(Draw {
                    coord,
                    symbol: character.symbol.clone(),
                    color,
                    bold: pull > 0.65,
                });
            }

            for (ring_position, _) in ring_indices.iter().enumerate() {
                let angle = ring_position as f64 / ring_count as f64 * PI * 2.0
                    + global * PI * 5.0;
                let pulse = 1.0 + (global * PI * 12.0).sin() * 0.08;
                let color_index =
                    (ring_position * 5 + frame * 2) % blackhole_gradient.len();

                draws.push(Draw {
                    coord: ring_coord(center, radius * pulse, angle),
                    symbol: "*".to_owned(),
                    color: blackhole_gradient[color_index],
                    bold: true,
                });
            }

            draws.push(Draw {
                coord: center,
                symbol: "●".to_owned(),
                color: Color::new(0x08, 0x00, 0x10),
                bold: true,
            });

            frames.push(render_frame(width, height, draws));
        }

        // The event horizon collapses into a single point.
        for frame in 0..20 {
            let progress = easing::in_quint((frame + 1) as f64 / 20.0);
            let current_radius = radius * (1.0 - progress);
            let mut draws = Vec::with_capacity(ring_count + 1);

            for (ring_position, _) in ring_indices.iter().enumerate() {
                let angle = ring_position as f64 / ring_count as f64 * PI * 2.0
                    + progress * PI * 4.0;
                let color_index =
                    (ring_position * 3 + frame * 2) % blackhole_gradient.len();

                draws.push(Draw {
                    coord: ring_coord(center, current_radius, angle),
                    symbol: "*".to_owned(),
                    color: blackhole_gradient[color_index],
                    bold: true,
                });
            }

            draws.push(Draw {
                coord: center,
                symbol: if frame > 15 { "◆" } else { "●" }.to_owned(),
                color: blackhole_gradient
                    [(frame * 2).min(blackhole_gradient.len() - 1)],
                bold: true,
            });

            frames.push(render_frame(width, height, draws));
        }

        // The singularity bursts outward.
        let particle_count = 24;
        for frame in 0..18 {
            let progress = easing::out_expo((frame + 1) as f64 / 18.0);
            let maximum_radius = width.max(height * 2) as f64 * 0.65;
            let mut draws = Vec::with_capacity(particle_count + 1);

            for particle in 0..particle_count {
                let jitter = mix(particle as u64 * 7919);
                let angle = particle as f64 / particle_count as f64 * PI * 2.0
                    + ((jitter & 255) as f64 / 255.0 - 0.5) * 0.25;
                let distance = maximum_radius
                    * progress
                    * (0.55 + ((jitter >> 8) & 255) as f64 / 512.0);
                let coord = ring_coord(center, distance, angle);
                let color_index = ((1.0 - progress)
                    * (blackhole_gradient.len() - 1) as f64)
                    as usize;

                draws.push(Draw {
                    coord,
                    symbol: if particle % 3 == 0 { "✦" } else { "*" }.to_owned(),
                    color: blackhole_gradient[color_index],
                    bold: true,
                });
            }

            draws.push(Draw {
                coord: center,
                symbol: "✹".to_owned(),
                color: Color::new(0xff, 0xff, 0xe8),
                bold: true,
            });

            frames.push(render_frame(width, height, draws));
        }

        // Every consumed character is restored to its original coordinate.
        for frame in 0..48 {
            let global = (frame + 1) as f64 / 48.0;
            let mut draws = Vec::with_capacity(characters.len());

            for character in &characters {
                let delay = (character.hash & 15) as f64 / 100.0;
                let local = ((global - delay) / (1.0 - delay)).clamp(0.0, 1.0);
                if local <= 0.0 {
                    continue;
                }

                let progress = easing::out_expo(local);
                let coord = center.lerp(character.input_coord, progress);
                let hot = Color::new(0xff, 0xf3, 0xc0);
                let color = hot.lerp(character.final_color, progress);

                draws.push(Draw {
                    coord,
                    symbol: character.symbol.clone(),
                    color,
                    bold: local < 0.72,
                });
            }

            frames.push(render_frame(width, height, draws));
        }

        // Hold the fully restored, colored text briefly.
        for frame in 0..8 {
            let mut draws = Vec::with_capacity(characters.len());

            for character in &characters {
                let pulse = ((character.hash as usize + frame) % 7) as f64 / 35.0;
                draws.push(Draw {
                    coord: character.input_coord,
                    symbol: character.symbol.clone(),
                    color: character
                        .final_color
                        .lerp(Color::new(0xff, 0xff, 0xff), pulse),
                    bold: frame < 3,
                });
            }

            frames.push(render_frame(width, height, draws));
        }

        frames
    }
}

#[derive(Clone)]
struct BlackholeCharacter {
    symbol: String,
    input_coord: Coord,
    final_color: Color,
    hash: u64,
}

struct Draw {
    coord: Coord,
    symbol: String,
    color: Color,
    bold: bool,
}

fn ring_coord(center: Coord, radius: f64, angle: f64) -> Coord {
    Coord::new(
        (center.column as f64 + radius * angle.cos()).round() as i32,
        (center.row as f64 + radius * angle.sin() / 2.0).round() as i32,
    )
}

fn render_frame(width: usize, height: usize, draws: Vec<Draw>) -> String {
    let mut canvas = Canvas::new(width, height);
    let fallback_color = Color::new(0x76, 0x18, 0xb8);
    let mut drew_visible_cell = false;

    for draw in draws {
        let style = Style {
            bold: draw.bold,
            ..Style::with_colors(ColorPair::new(Some(draw.color), None))
        };

        if canvas.set(draw.coord, draw.symbol, style) {
            drew_visible_cell = true;
        }
    }

    if !drew_visible_cell {
        canvas.set(
            Coord::new(0, 0),
            " ",
            Style::with_colors(ColorPair::new(Some(fallback_color), None)),
        );
    }

    canvas.render()
}

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
