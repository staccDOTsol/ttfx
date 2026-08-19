
use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient, Style};

const LAUNCHER_COUNT: usize = 4;
const VOLLEY_SIZE: usize = 3;
const VOLLEY_DELAY: usize = 7;
const SHOT_STAGGER: usize = 1;
const FLIGHT_SPEED: f64 = 0.75;
const FINAL_HOLD_FRAMES: usize = 4;

#[derive(Clone, Debug)]
struct VolleyCharacter {
    symbol: String,
    target: Coord,
    launcher: usize,
    launch_frame: usize,
    duration: usize,
}

pub struct Orbittingvolley;

impl Orbittingvolley {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Orbittingvolley {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Orbittingvolley {
    fn name(&self) -> &str {
        "orbittingvolley"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines = input_lines(input);
        let width = lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let final_gradient = Gradient::new(
            [
                Color::new(255, 180, 0),
                Color::new(255, 70, 120),
                Color::new(120, 70, 255),
                Color::new(0, 210, 255),
            ],
            256,
        )
        .colors();

        let launcher_gradient = Gradient::new(
            [
                Color::new(0, 255, 210),
                Color::new(40, 130, 255),
                Color::new(180, 60, 255),
                Color::new(255, 60, 120),
                Color::new(255, 220, 40),
                Color::new(0, 255, 210),
            ],
            256,
        )
        .colors();

        let mut magazines: [Vec<(String, Coord)>; LAUNCHER_COUNT] =
            std::array::from_fn(|_| Vec::new());

        for (row, line) in lines.iter().enumerate() {
            for (column, symbol) in line.chars().enumerate() {
                if symbol.is_whitespace() {
                    continue;
                }

                let target = Coord::new(column as i32, row as i32);
                let launcher = target_launcher(target, width, height);
                magazines[launcher].push((symbol.to_string(), target));
            }
        }

        for (launcher, magazine) in magazines.iter_mut().enumerate() {
            let corner = launcher_corner(launcher, width, height);
            magazine.sort_by_key(|(_, target)| {
                (
                    target.manhattan_distance_to(corner),
                    target.row,
                    target.column,
                )
            });
        }

        let perimeter = perimeter_length(width, height);
        let orbit_speed = if perimeter <= 4 {
            1
        } else {
            (perimeter / 48).max(1)
        };

        let mut volley_characters = Vec::new();
        let mut last_arrival = 0;

        for (launcher, magazine) in magazines.into_iter().enumerate() {
            for (index, (symbol, target)) in magazine.into_iter().enumerate() {
                let launch_frame = (index / VOLLEY_SIZE) * VOLLEY_DELAY
                    + (index % VOLLEY_SIZE) * SHOT_STAGGER;
                let start =
                    launcher_position(launcher, launch_frame, orbit_speed, width, height);
                let distance = start.distance_to(target);
                let duration = (distance / FLIGHT_SPEED).ceil().max(1.0) as usize;

                last_arrival = last_arrival.max(launch_frame + duration);
                volley_characters.push(VolleyCharacter {
                    symbol,
                    target,
                    launcher,
                    launch_frame,
                    duration,
                });
            }
        }

        let animation_frames = if volley_characters.is_empty() {
            perimeter.max(8).min(24)
        } else {
            last_arrival + 1
        };
        let total_frames = animation_frames + FINAL_HOLD_FRAMES;
        let mut frames = Vec::with_capacity(total_frames);

        for frame_index in 0..total_frames {
            let mut canvas = Canvas::new(width, height);
            let show_launchers = frame_index < animation_frames;

            // Settled characters form the final text beneath active shots.
            for character in &volley_characters {
                if frame_index < character.launch_frame + character.duration {
                    continue;
                }

                let color = coordinate_color(
                    character.target,
                    width,
                    height,
                    &final_gradient,
                );
                canvas.set(
                    character.target,
                    character.symbol.clone(),
                    colored_style(color, false),
                );
            }

            // Characters inherit the perimeter gradient while in flight.
            for character in &volley_characters {
                if frame_index < character.launch_frame
                    || frame_index >= character.launch_frame + character.duration
                {
                    continue;
                }

                let elapsed = frame_index - character.launch_frame;
                let raw_progress = elapsed as f64 / character.duration as f64;
                let progress = easing::out_sine(raw_progress);
                let start = launcher_position(
                    character.launcher,
                    character.launch_frame,
                    orbit_speed,
                    width,
                    height,
                );
                let position = start.lerp(character.target, progress);
                let color =
                    coordinate_color(position, width, height, &launcher_gradient);

                canvas.set(
                    position,
                    character.symbol.clone(),
                    colored_style(color, true),
                );
            }

            if show_launchers {
                for launcher in 0..LAUNCHER_COUNT {
                    let position = launcher_position(
                        launcher,
                        frame_index,
                        orbit_speed,
                        width,
                        height,
                    );
                    let color = coordinate_color(
                        position,
                        width,
                        height,
                        &launcher_gradient,
                    );

                    canvas.set(position, "◉", colored_style(color, true));
                }
            }

            frames.push(canvas.render());
        }

        frames
    }
}

fn input_lines(input: &str) -> Vec<String> {
    let input = input.trim_end_matches(&['\r', '\n'][..]);

    if input.is_empty() {
        vec![String::new()]
    } else {
        input
            .split('\n')
            .map(|line| line.trim_end_matches('\r').to_owned())
            .collect()
    }
}

fn colored_style(color: Color, bold: bool) -> Style {
    Style {
        bold,
        ..Style::with_colors(ColorPair::new(Some(color), None))
    }
}

fn target_launcher(target: Coord, width: usize, height: usize) -> usize {
    let right_half = target.column as usize >= width.div_ceil(2);
    let bottom_half = target.row as usize >= height.div_ceil(2);

    match (right_half, bottom_half) {
        (false, false) => 0,
        (true, false) => 1,
        (true, true) => 2,
        (false, true) => 3,
    }
}

fn launcher_corner(launcher: usize, width: usize, height: usize) -> Coord {
    let right = width.saturating_sub(1) as i32;
    let bottom = height.saturating_sub(1) as i32;

    match launcher % LAUNCHER_COUNT {
        0 => Coord::new(0, 0),
        1 => Coord::new(right, 0),
        2 => Coord::new(right, bottom),
        _ => Coord::new(0, bottom),
    }
}

fn perimeter_length(width: usize, height: usize) -> usize {
    match (width, height) {
        (1, 1) => 1,
        (1, height) => height,
        (width, 1) => width,
        (width, height) => 2 * width + 2 * height - 4,
    }
}

fn launcher_position(
    launcher: usize,
    frame: usize,
    orbit_speed: usize,
    width: usize,
    height: usize,
) -> Coord {
    let perimeter = perimeter_length(width, height);
    if perimeter <= 1 {
        return Coord::new(0, 0);
    }

    let quarter_offset = launcher * perimeter / LAUNCHER_COUNT;
    let distance = (quarter_offset + frame * orbit_speed) % perimeter;

    perimeter_coord(distance, width, height)
}

fn perimeter_coord(distance: usize, width: usize, height: usize) -> Coord {
    if width == 1 {
        return Coord::new(0, (distance % height) as i32);
    }
    if height == 1 {
        return Coord::new((distance % width) as i32, 0);
    }

    let top_length = width - 1;
    let right_length = height - 1;
    let bottom_length = width - 1;

    if distance <= top_length {
        return Coord::new(distance as i32, 0);
    }

    let distance = distance - top_length;
    if distance <= right_length {
        return Coord::new((width - 1) as i32, distance as i32);
    }

    let distance = distance - right_length;
    if distance <= bottom_length {
        return Coord::new((width - 1 - distance) as i32, (height - 1) as i32);
    }

    let distance = distance - bottom_length;
    Coord::new(0, (height - 1 - distance) as i32)
}

fn coordinate_color(
    coord: Coord,
    width: usize,
    height: usize,
    colors: &[Color],
) -> Color {
    if colors.is_empty() {
        return Color::new(255, 255, 255);
    }

    let column_progress = if width <= 1 {
        0.5
    } else {
        coord.column.clamp(0, width as i32 - 1) as f64
            / (width - 1) as f64
    };
    let row_progress = if height <= 1 {
        0.5
    } else {
        coord.row.clamp(0, height as i32 - 1) as f64
            / (height - 1) as f64
    };

    let progress = (column_progress + row_progress) / 2.0;
    let index =
        (progress * (colors.len() - 1) as f64).round() as usize;

    colors[index.min(colors.len() - 1)]
}
