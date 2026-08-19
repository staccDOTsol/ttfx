use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Bouncyballs;

impl Bouncyballs {
    pub fn new() -> Self {
        Bouncyballs
    }
}

impl Effect for Bouncyballs {
    fn name(&self) -> &str {
        "bouncyballs"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let input = if input.trim().is_empty() {
            "TerminalTextEffects"
        } else {
            input
        };

        let mut chars: Vec<(Coord, char)> = Vec::new();
        let mut width: usize = 0;
        let mut height: usize = 0;

        for (row, line) in input.lines().enumerate() {
            width = width.max(line.chars().count());
            height = row + 1;
            for (col, ch) in line.chars().enumerate() {
                chars.push((Coord::new(col as i32, row as i32), ch));
            }
        }

        if width == 0 || height == 0 || chars.is_empty() {
            return Vec::new();
        }

        let mut terminal = Terminal::new(width as u16, height as u16);

        for (id, (coord, symbol)) in chars.iter().enumerate() {
            terminal.add_character(EffectCharacter::new(id as u32, *coord, *symbol));
        }

        // Start with dim, clearly visible text; bouncing balls will recolor it.
        for ch in terminal.characters.iter_mut() {
            ch.visible = true;
            ch.color_pair = ColorPair::new(Color::new(80, 80, 80), Color::new(0, 0, 0));
        }

        let mut balls = create_balls(width as i32, height as i32);
        let total_frames = 200;
        let mut frames = Vec::with_capacity(total_frames);

        for _ in 0..total_frames {
            update_balls(&mut balls, width as i32, height as i32);

            for ch in terminal.characters.iter_mut() {
                let char_pos = ch.position;
                let mut best_dist = f64::MAX;
                let mut best_color: Option<Color> = None;

                for ball in balls.iter() {
                    let dx = char_pos.x as f64 - ball.pos_x;
                    let dy = char_pos.y as f64 - ball.pos_y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < ball.radius && dist < best_dist {
                        best_dist = dist;
                        let t = (dist / ball.radius).clamp(0.0, 1.0);
                        best_color = Some(ball_gradient(ball.color).color_at(t));
                    }
                }

                if let Some(color) = best_color {
                    ch.color_pair = ColorPair::new(color, Color::new(10, 10, 10));
                    ch.bold = best_dist < 1.5;
                }
            }

            frames.push(terminal.render_frame());
        }

        frames
    }
}

struct Ball {
    pos_x: f64,
    pos_y: f64,
    vx: f64,
    vy: f64,
    color: Color,
    radius: f64,
}

fn create_balls(width: i32, height: i32) -> Vec<Ball> {
    let palette = [
        Color::new(255, 85, 85),
        Color::new(255, 184, 77),
        Color::new(255, 255, 85),
        Color::new(128, 255, 128),
        Color::new(85, 221, 255),
        Color::new(170, 128, 255),
        Color::new(255, 85, 255),
    ];

    let target_count = ((width as usize + height as usize) / 12 + 3).min(10);
    let mut balls = Vec::with_capacity(target_count);

    for i in 0..target_count {
        let fx = (i + 1) as f64 / (target_count + 1) as f64;
        let fy = ((i as f64 * 2.0) + 1.0) / (2.0 * target_count as f64);
        let x = fx * (width.max(2) - 1) as f64;
        let y = fy * (height.max(2) - 1) as f64;

        let angle = i as f64 * 2.399963;
        let speed = 0.35 + (i % 3) as f64 * 0.12;

        balls.push(Ball {
            pos_x: x,
            pos_y: y,
            vx: speed * angle.cos(),
            vy: speed * angle.sin(),
            color: palette[i % palette.len()],
            radius: 4.5 + (i % 3) as f64,
        });
    }

    balls
}

fn update_balls(balls: &mut [Ball], width: i32, height: i32) {
    let max_x = (width.max(2) - 1) as f64;
    let max_y = (height.max(2) - 1) as f64;

    for ball in balls.iter_mut() {
        ball.pos_x += ball.vx;
        ball.pos_y += ball.vy;

        if ball.pos_x <= 0.0 {
            ball.pos_x = 0.0;
            ball.vx = ball.vx.abs();
        } else if ball.pos_x >= max_x {
            ball.pos_x = max_x;
            ball.vx = -ball.vx.abs();
        }

        if ball.pos_y <= 0.0 {
            ball.pos_y = 0.0;
            ball.vy = ball.vy.abs();
        } else if ball.pos_y >= max_y {
            ball.pos_y = max_y;
            ball.vy = -ball.vy.abs();
        }
    }
}

fn ball_gradient(color: Color) -> Gradient {
    Gradient::new(vec![(0.0, color), (1.0, Color::new(40, 40, 40))])
}
