use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, Gradient};

pub struct Waves;

impl Waves {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Waves {
    fn name(&self) -> &str {
        "waves"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let width = input
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let height = input.lines().count().max(1);
        let mut term = Terminal::from_input(input, width, height);

        let wave_stops = vec![
            Color::rgb(0x00, 0x4b, 0x8d),
            Color::rgb(0x00, 0x8c, 0xba),
            Color::rgb(0x58, 0xc4, 0xdd),
            Color::rgb(0xff, 0xff, 0xff),
            Color::rgb(0x58, 0xc4, 0xdd),
            Color::rgb(0x00, 0x8c, 0xba),
        ];
        let spectrum = Gradient::new(wave_stops, 24).colors();
        let n = spectrum.len().max(1);

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        let mut frames = Vec::new();
        let total = (width + height + n + 8) as i32;
        for tick in 0..total {
            for ch in term.get_characters_mut() {
                let phase = (ch.input_coord.column + ch.input_coord.row + tick) as usize;
                let color = spectrum[phase % n];
                ch.animation.set_appearance(ch.input_symbol, Some(color));
                let bob = ((tick + ch.input_coord.column) as f64 * 0.35).sin();
                ch.current_coord = Coord {
                    column: ch.input_coord.column,
                    row: ch.input_coord.row + if bob > 0.6 { 1 } else { 0 },
                };
            }
            let raw = term.get_formatted_output_string();
            let mut painted = String::new();
            for ch in term.get_characters() {
                if !ch.is_visible {
                    continue;
                }
                let c = ch
                    .animation
                    .current_character_visual
                    .colors
                    .and_then(|p| p.fg)
                    .unwrap_or(Color::rgb(88, 196, 221));
                painted.push_str(&format!(
                    "\x1b[{};{}H\x1b[38;2;{};{};{}m{}",
                    ch.current_coord.row + 1,
                    ch.current_coord.column + 1,
                    c.r,
                    c.g,
                    c.b,
                    ch.animation.current_character_visual.symbol
                ));
            }
            painted.push_str("\x1b[0m");
            painted.push_str(&raw);
            frames.push(painted);
        }
        frames
    }
}
