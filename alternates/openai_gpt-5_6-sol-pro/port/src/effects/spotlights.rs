
use super::Effect;
use crate::engine::Canvas;
use crate::utils::easing;
use crate::utils::{Color, Gradient, Style};

#[derive(Clone, Copy, Debug, Default)]
pub struct Spotlights;

impl Spotlights {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Spotlights {
    fn name(&self) -> &str {
        "spotlights"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        const SEARCH_FRAMES: usize = 54;
        const CONVERGE_FRAMES: usize = 20;

        let lines = parse_input(input);
        let width = lines
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(1);
        let height = lines.len().max(1);

        let final_colors = Gradient::new(
            [
                Color::new(0xab, 0x48, 0xff),
                Color::new(0xe7, 0xb2, 0xb2),
                Color::new(0xff, 0xfe, 0xbd),
            ],
            height,
        )
        .colors();

        let dark = Color::new(12, 8, 24);
        let spotlight = Color::new(255, 255, 238);
        let radius = ((width as f64).max(height as f64 * 2.0) * 0.24)
            .max(2.5);

        let routes: [&[(f64, f64)]; 3] = [
            &[
                (-0.20, -0.10),
                (0.20, 0.24),
                (0.78, 0.18),
                (0.36, 0.76),
                (0.86, 0.72),
                (0.22, 0.46),
            ],
            &[
                (1.20, -0.12),
                (0.72, 0.34),
                (0.24, 0.16),
                (0.66, 0.82),
                (0.14, 0.70),
                (0.76, 0.42),
            ],
            &[
                (0.48, 1.20),
                (0.46, 0.62),
                (0.88, 0.46),
                (0.18, 0.36),
                (0.52, 0.12),
                (0.52, 0.78),
            ],
        ];

        let mut frames = Vec::with_capacity(SEARCH_FRAMES + CONVERGE_FRAMES);

        for frame_index in 0..SEARCH_FRAMES {
            let progress = if SEARCH_FRAMES <= 1 {
                1.0
            } else {
                frame_index as f64 / (SEARCH_FRAMES - 1) as f64
            };

            let positions = [
                route_position(routes[0], progress, width, height),
                route_position(routes[1], progress, width, height),
                route_position(routes[2], progress, width, height),
            ];

            frames.push(render_frame(
                &lines,
                width,
                height,
                &final_colors,
                dark,
                spotlight,
                &positions,
                radius,
                0.0,
            ));
        }

        let route_ends = [
            route_position(routes[0], 1.0, width, height),
            route_position(routes[1], 1.0, width, height),
            route_position(routes[2], 1.0, width, height),
        ];
        let center = (
            (width.saturating_sub(1)) as f64 / 2.0,
            (height.saturating_sub(1)) as f64 / 2.0,
        );

        for frame_index in 1..=CONVERGE_FRAMES {
            let linear_progress = frame_index as f64 / CONVERGE_FRAMES as f64;
            let progress = easing::in_out_quad(linear_progress);

            let positions = route_ends.map(|start| {
                (
                    start.0 + (center.0 - start.0) * progress,
                    start.1 + (center.1 - start.1) * progress,
                )
            });

            frames.push(render_frame(
                &lines,
                width,
                height,
                &final_colors,
                dark,
                spotlight,
                &positions,
                radius * (1.0 + progress * 0.35),
                easing::in_out_sine(linear_progress),
            ));
        }

        frames
    }
}

#[allow(clippy::too_many_arguments)]
fn render_frame(
    lines: &[Vec<char>],
    width: usize,
    height: usize,
    final_colors: &[Color],
    dark: Color,
    spotlight: Color,
    positions: &[(f64, f64); 3],
    radius: f64,
    reveal: f64,
) -> String {
    let mut canvas = Canvas::new(width, height);
    canvas.fill(
        " ",
        Style {
            foreground: Some(dark),
            ..Style::default()
        },
    );

    for (row, line) in lines.iter().enumerate() {
        let final_color = final_colors
            .get(row)
            .copied()
            .unwrap_or(Color::new(255, 254, 189));

        for (column, symbol) in line.iter().enumerate() {
            let x = column as f64;
            let y = row as f64;

            let illumination = positions.iter().fold(0.0_f64, |current, position| {
                let dx = x - position.0;
                let dy = (y - position.1) * 2.0;
                let distance = dx.hypot(dy);
                let intensity = (1.0 - distance / radius).clamp(0.0, 1.0);
                let softened = intensity * intensity * (3.0 - 2.0 * intensity);
                current.max(softened)
            });

            let shadow_color = dark.lerp(final_color, 0.16);
            let illuminated_color =
                shadow_color.lerp(spotlight, illumination * 0.94);
            let color = illuminated_color.lerp(final_color, reveal);

            canvas.set(
                crate::utils::Coord::new(column as i32, row as i32),
                symbol.to_string(),
                Style {
                    foreground: Some(color),
                    bold: illumination > 0.82 && reveal < 0.9,
                    ..Style::default()
                },
            );
        }
    }

    canvas.render()
}

fn route_position(
    route: &[(f64, f64)],
    progress: f64,
    width: usize,
    height: usize,
) -> (f64, f64) {
    if route.len() == 1 {
        return scale_position(route[0], width, height);
    }

    let scaled = progress.clamp(0.0, 1.0) * (route.len() - 1) as f64;
    let segment = (scaled.floor() as usize).min(route.len() - 2);
    let local_progress = easing::in_out_sine(scaled - segment as f64);
    let start = route[segment];
    let end = route[segment + 1];

    scale_position(
        (
            start.0 + (end.0 - start.0) * local_progress,
            start.1 + (end.1 - start.1) * local_progress,
        ),
        width,
        height,
    )
}

fn scale_position(
    position: (f64, f64),
    width: usize,
    height: usize,
) -> (f64, f64) {
    (
        position.0 * width as f64 - 0.5,
        position.1 * height as f64 - 0.5,
    )
}

fn parse_input(input: &str) -> Vec<Vec<char>> {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let normalized = normalized.trim_end_matches('\n');

    if normalized.is_empty() {
        return vec![Vec::new()];
    }

    normalized
        .split('\n')
        .map(|line| {
            let mut characters = Vec::new();

            for character in line.chars() {
                if character == '\t' {
                    characters.extend([' ', ' ', ' ', ' ']);
                } else {
                    characters.push(character);
                }
            }

            characters
        })
        .collect()
}
