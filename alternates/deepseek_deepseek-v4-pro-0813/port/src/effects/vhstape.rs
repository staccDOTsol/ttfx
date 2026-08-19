use super::Effect;
use crate::engine::canvas::Canvas;
use crate::engine::character::EffectCharacter;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};

pub struct Vhstape;

impl Vhstape {
    pub fn new() -> Self {
        Vhstape
    }
}

struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Rng { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state
    }

    fn range(&mut self, low: usize, high: usize) -> usize {
        if high <= low {
            return low;
        }
        low + (self.next_u32() as usize % (high - low))
    }
}

impl Effect for Vhstape {
    fn name(&self) -> &str {
        "vhstape"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let line_count = lines.len().max(1);
        let max_line_length = lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0)
            .max(1);

        // Generous padding for chromatic fringing and tracking bands.
        let canvas_width = (max_line_length + 8).max(12) as u16;
        let canvas_height = (line_count + 6).max(8) as u16;
        let canvas = Canvas::new(canvas_width, canvas_height);

        let mut originals: Vec<(Coord, char)> = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                originals.push((Coord::new(col as i32 + 4, row as i32 + 3), ch));
            }
        }

        const VHS_COLORS: [Color; 4] = [
            Color { r: 220, g: 220, b: 220 },
            Color { r: 190, g: 255, b: 230 },
            Color { r: 210, g: 220, b: 255 },
            Color { r: 255, g: 210, b: 210 },
        ];
        const NOISE_COLORS: [Color; 5] = [
            Color { r: 90, g: 90, b: 90 },
            Color { r: 120, g: 120, b: 120 },
            Color { r: 60, g: 80, b: 60 },
            Color { r: 80, g: 60, b: 90 },
            Color { r: 70, g: 70, b: 80 },
        ];
        const NOISE_SYMBOLS: [char; 6] = ['.', ',', '`', '*', '+', '~'];

        let mut rng = Rng::new(0x5EED_C0DE);
        let frame_count = 24 + (originals.len() % 8);
        let mut frames = Vec::with_capacity(frame_count);

        for frame_index in 0..frame_count {
            let mut chars: Vec<EffectCharacter> = Vec::new();
            let mut next_id: u32 = 0;

            // Static/noise behind the text so it reads as scrambled tape grain.
            let noise_count = (canvas_width as usize / 2).max(30);
            for _ in 0..noise_count {
                let x = rng.range(0, canvas_width as usize) as i32;
                let y = rng.range(0, canvas_height as usize) as i32;
                let symbol = NOISE_SYMBOLS[rng.range(0, NOISE_SYMBOLS.len())];
                let color = NOISE_COLORS[rng.range(0, NOISE_COLORS.len())];
                let mut c = EffectCharacter::new(next_id, Coord::new(x, y), symbol);
                c.color_pair = ColorPair::new(color, Color::BLACK);
                c.dim = true;
                c.visible = true;
                chars.push(c);
                next_id += 1;
            }

            // Chromatic-aberration-style text: red/blue fringe first, then main.
            let base_color = VHS_COLORS[frame_index % VHS_COLORS.len()];
            for (orig_coord, symbol) in &originals {
                let jitter_x = if rng.range(0, 100) < 18 {
                    rng.range(0, 3) as i32 - 1
                } else {
                    0
                };

                let red_pos = Coord::new(orig_coord.x - 1, orig_coord.y);
                let blue_pos = Coord::new(orig_coord.x + 1, orig_coord.y);
                let main_pos = Coord::new(orig_coord.x + jitter_x, orig_coord.y);

                let mut red = EffectCharacter::new(next_id, red_pos, *symbol);
                red.color_pair = ColorPair::new(Color::RED, Color::BLACK);
                red.dim = true;
                red.visible = true;
                chars.push(red);
                next_id += 1;

                let mut blue = EffectCharacter::new(next_id, blue_pos, *symbol);
                blue.color_pair = ColorPair::new(Color::BLUE, Color::BLACK);
                blue.dim = true;
                blue.visible = true;
                chars.push(blue);
                next_id += 1;

                let mut main = EffectCharacter::new(next_id, main_pos, *symbol);
                main.color_pair = ColorPair::new(base_color, Color::BLACK);
                main.visible = true;
                chars.push(main);
                next_id += 1;
            }

            // Tracking band across a random row.
            let tracking_y = rng.range(0, canvas_height as usize) as i32;
            for x in 0..canvas_width as i32 {
                let mut band = EffectCharacter::new(next_id, Coord::new(x, tracking_y), '█');
                band.color_pair = ColorPair::new(Color::WHITE, Color::BLACK);
                band.bold = true;
                band.visible = true;
                chars.push(band);
                next_id += 1;
            }

            frames.push(canvas.render(&chars));
        }

        frames
    }
}
