//! Matrix digital-rain effect (port of terminaltexteffects/effects/effect_matrix.py).
//!
//! Phase 1: green "digital rain" streams fall down every column of the canvas,
//! with a bright highlight head and a gradient tail of churning katakana/symbol
//! glyphs. Phase 2: the rain resolves into the input text — each character
//! cycles a few rain glyphs, flashes the highlight color, then fades to the
//! final matrix green (mirroring the Python rain -> highlight -> final flow).

use super::Effect;
use crate::engine::animation::CharacterVisual;
use crate::engine::terminal::{Terminal, TerminalConfig};
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

/// Rain glyph pool (subset of the Python `rain_symbols` default: digits,
/// punctuation, and half-width katakana).
const RAIN_SYMBOLS: &[char] = &[
    '2', '5', '9', '8', 'Z', '*', ')', ':', '.', '"', '=', '+', '-', '|', '_', 'c',
    'ｦ', 'ｱ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ',
    'ﾀ', 'ﾂ', 'ﾃ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾊ', 'ﾋ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ',
    'ﾓ', 'ﾔ', 'ﾕ', 'ﾗ', 'ﾘ', 'ﾜ',
];

/// Number of ticks spent in the pure rain phase (analog of `rain_time`).
const RAIN_FRAMES: usize = 110;
/// Safety cap for the resolve phase.
const MAX_RESOLVE_FRAMES: usize = 500;

/// Small deterministic xorshift* PRNG (the crate has no rand dependency).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(if seed == 0 { 0x9e3779b97f4a7c15 } else { seed })
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as u32
    }

    /// Random i32 in `lo..hi` (exclusive upper bound).
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() % ((hi - lo) as u32)) as i32
    }

    fn choice<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[(self.next_u32() as usize) % items.len()]
    }
}

/// One falling rain stream in a single column.
struct Stream {
    /// Ticks to wait before this stream starts (or restarts) falling.
    delay: i32,
    /// Canvas row of the stream head (row 1 is the bottom; head moves down).
    head_row: i32,
    /// Number of glyphs in the stream, head included.
    length: i32,
    /// Ticks between one-row advances (analog of `rain_fall_delay_range`).
    fall_delay: i32,
    ticks_since_fall: i32,
}

impl Stream {
    fn spawn(rng: &mut Rng, height: i32, initial: bool) -> Self {
        let delay = if initial {
            rng.range(0, 45)
        } else {
            rng.range(5, 30)
        };
        Stream {
            delay,
            head_row: height + rng.range(0, height.max(2)),
            length: rng.range((height / 4).max(3), height.max(4) + 1),
            fall_delay: rng.range(1, 4),
            ticks_since_fall: 0,
        }
    }

    fn tick(&mut self, rng: &mut Rng, height: i32) {
        if self.delay > 0 {
            self.delay -= 1;
            return;
        }
        self.ticks_since_fall += 1;
        if self.ticks_since_fall >= self.fall_delay {
            self.ticks_since_fall = 0;
            self.head_row -= 1;
        }
        // Entire stream has scrolled off the bottom: recycle it.
        if self.head_row + self.length < 1 {
            *self = Stream::spawn(rng, height, false);
        }
    }
}

pub struct Matrix;

impl Matrix {
    pub fn new() -> Self {
        Matrix
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Effect for Matrix {
    fn name(&self) -> &str {
        "matrix"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let mut terminal = Terminal::new(input, TerminalConfig::default());
        let width = terminal.canvas.width;
        let height = terminal.canvas.height;
        let mut rng = Rng::new(0x7454_4658_6d61_7472); // fixed seed: deterministic output

        // Colors, mirroring the Python defaults:
        // highlight_color "dbfcfc", rain gradient "92be92" -> "185318",
        // final gradient stop "389c38".
        let highlight_color = Color::from_hex("dbfcfc").expect("valid hex");
        let rain_bright = Color::from_hex("92be92").expect("valid hex");
        let rain_dark = Color::from_hex("185318").expect("valid hex");
        let final_color = Color::from_hex("389c38").expect("valid hex");
        let rain_gradient = Gradient::new(&[rain_bright, rain_dark], 12);
        let resolve_gradient = Gradient::new(&[highlight_color, final_color], 6);

        let mut frames: Vec<String> = Vec::new();

        // ---- Phase 1: digital rain over the whole canvas ----------------
        let mut streams: Vec<Stream> = (0..width)
            .map(|_| Stream::spawn(&mut rng, height, true))
            .collect();

        // Per-cell glyph grid; a slice of cells churns to new glyphs each tick,
        // approximating the Python looping rain scenes.
        let cell_count = (width as usize) * (height as usize);
        let mut glyphs: Vec<char> = (0..cell_count)
            .map(|_| *rng.choice(RAIN_SYMBOLS))
            .collect();
        let churn_per_frame = (cell_count / 12).max(1);

        for _ in 0..RAIN_FRAMES {
            for cell in 0..churn_per_frame {
                let _ = cell;
                let idx = (rng.next_u32() as usize) % cell_count;
                glyphs[idx] = *rng.choice(RAIN_SYMBOLS);
            }

            terminal.canvas.clear();
            for (col_index, stream) in streams.iter_mut().enumerate() {
                stream.tick(&mut rng, height);
                if stream.delay > 0 {
                    continue;
                }
                let column = col_index as i32 + 1;
                for d in 0..stream.length {
                    // Head at the bottom of the stream; tail trails above it.
                    let row = stream.head_row + d;
                    if row < 1 || row > height {
                        continue;
                    }
                    let coord = Coord::new(column, row);
                    let glyph_idx =
                        ((row - 1) as usize) * (width as usize) + (column - 1) as usize;
                    let symbol = glyphs[glyph_idx];
                    let visual = if d == 0 {
                        let mut head =
                            CharacterVisual::new(symbol, Some(ColorPair::fg_only(highlight_color)));
                        head.bold = true;
                        head
                    } else {
                        let fraction = d as f64 / (stream.length.max(2) - 1) as f64;
                        let color = rain_gradient
                            .get_color_at_fraction(fraction)
                            .unwrap_or(rain_dark);
                        CharacterVisual::new(symbol, Some(ColorPair::fg_only(color)))
                    };
                    terminal.canvas.set_cell(coord, visual);
                }
            }
            frames.push(terminal.canvas.to_frame_string());
        }

        // ---- Phase 2: the rain resolves into the input text -------------
        for character in terminal.get_characters_mut() {
            character.is_visible = true;
            let input_symbol = character.input_symbol;

            // Pre-pick the randomized lead-in so the scene borrow stays tidy.
            let lead_count = rng.range(2, 7);
            let mut lead_frames: Vec<(char, u32, Color)> = Vec::new();
            for _ in 0..lead_count {
                let symbol = *rng.choice(RAIN_SYMBOLS);
                let duration = rng.range(2, 9) as u32;
                let color = *rng.choice(&rain_gradient.spectrum);
                lead_frames.push((symbol, duration, color));
            }

            let scene = character.animation.new_scene("resolve", false);
            for (symbol, duration, color) in lead_frames {
                scene.add_frame(symbol, duration, Some(ColorPair::fg_only(color)));
            }
            // Highlight flash on the real symbol, then fade to the final green.
            scene.add_frame(input_symbol, 3, Some(ColorPair::fg_only(highlight_color)));
            for color in &resolve_gradient.spectrum {
                scene.add_frame(input_symbol, 2, Some(ColorPair::fg_only(*color)));
            }
            scene.add_frame(input_symbol, 1, Some(ColorPair::fg_only(final_color)));
            character.animation.activate_scene("resolve");
        }

        let mut resolve_frames = 0usize;
        while terminal.is_active() && resolve_frames < MAX_RESOLVE_FRAMES {
            terminal.tick();
            frames.push(terminal.render_frame());
            resolve_frames += 1;
        }
        // Hold the fully resolved text for a beat.
        frames.push(terminal.render_frame());

        frames
    }
}
