use super::Effect;
use crate::engine::character::EffectCharacter;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};

struct Particle {
    id: u32,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    color: Color,
    life: f64,
    visible: bool,
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed | 1 }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn firework_colors() -> Vec<Color> {
    vec![
        Color::new(255, 0, 0),
        Color::new(255, 140, 0),
        Color::new(255, 255, 0),
        Color::new(0, 255, 0),
        Color::new(0, 200, 255),
        Color::new(0, 0, 255),
        Color::new(255, 0, 255),
    ]
}

fn fade_color(color: Color, life: f64) -> Color {
    let factor = life.max(0.0).min(1.0);
    Color::new(
        (color.r as f64 * factor) as u8,
        (color.g as f64 * factor) as u8,
        (color.b as f64 * factor) as u8,
    )
}

pub struct Fireworks;

impl Fireworks {
    pub fn new() -> Self {
        Fireworks
    }
}

impl Effect for Fireworks {
    fn name(&self) -> &str {
        "fireworks"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let max_line_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let width = max_line_len.max(30) as u16;
        let height = lines.len().max(10) as u16;

        let mut terminal = Terminal::new(width, height);

        let mut symbols: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
        if symbols.is_empty() {
            symbols = vec!['*'];
        }

        let launch_x = width / 2;
        let launch_y = height - 1;
        let explosion_y = (height / 3).max(1) as i32;

        let mut particles: Vec<Particle> = symbols
            .iter()
            .enumerate()
            .map(|(i, &sym)| {
                let id = i as u32;
                let character = EffectCharacter::new(
                    id,
                    Coord::new(launch_x as i32, launch_y as i32),
                    sym,
                );
                terminal.add_character(character);
                Particle {
                    id,
                    x: launch_x as f64,
                    y: launch_y as f64,
                    vx: 0.0,
                    vy: 0.0,
                    color: Color::WHITE,
                    life: 1.0,
                    visible: false,
                }
            })
            .collect();

        for p in &particles {
            terminal.set_character_visibility(p.id, p.visible);
        }
        let rocket_id = particles[0].id;
        terminal.set_character_visibility(rocket_id, true);

        let mut frames = Vec::new();

        // Launch the rocket from the bottom to the explosion height.
        let launch_frames = 12;
        for frame in 0..launch_frames {
            let t = frame as f64 / launch_frames as f64;
            let y = launch_y as f64 - (launch_y as f64 - explosion_y as f64) * t;
            if let Some(ch) = terminal.get_character_mut(rocket_id) {
                ch.position = Coord::new(launch_x as i32, y.round() as i32);
                ch.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
            }
            frames.push(terminal.render_frame());
        }

        // Explode: turn every character into a firework spark with a radial velocity.
        let mut rng = Rng::new(0x9E37_79B9_7F4A_7C15);
        let colors = firework_colors();
        for p in particles.iter_mut() {
            p.x = launch_x as f64;
            p.y = explosion_y as f64;
            p.visible = true;
            p.life = 1.0;

            let angle = rng.next_f64() * 2.0 * std::f64::consts::PI;
            let speed = 0.5 + rng.next_f64() * 2.0;
            p.vx = angle.cos() * speed;
            p.vy = angle.sin() * speed;
            p.color = colors[rng.next_u64() as usize % colors.len()];

            terminal.set_character_visibility(p.id, true);
        }

        // Particle flight, gravity, fading, and eventual removal.
        let post_frames = 60;
        for _ in 0..post_frames {
            for p in particles.iter_mut() {
                if !p.visible {
                    if let Some(ch) = terminal.get_character_mut(p.id) {
                        ch.visible = false;
                    }
                    continue;
                }

                p.vy += 0.06;
                p.x += p.vx;
                p.y += p.vy;
                p.life -= 0.03;

                if p.life <= 0.0
                    || p.y > terminal.canvas.height as f64 + 2.0
                    || p.y < -2.0
                    || p.x < -2.0
                    || p.x > terminal.canvas.width as f64 + 2.0
                {
                    p.visible = false;
                }

                let color = fade_color(p.color, p.life);
                if let Some(ch) = terminal.get_character_mut(p.id) {
                    ch.position = Coord::new(p.x.round() as i32, p.y.round() as i32);
                    ch.color_pair = ColorPair::new(color, Color::BLACK);
                    ch.visible = p.visible;
                }
            }

            let all_hidden = particles.iter().all(|p| !p.visible);
            frames.push(terminal.render_frame());
            if all_hidden {
                break;
            }
        }

        frames
    }
}
