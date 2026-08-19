use super::Effect;
use crate::engine::terminal::Terminal;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Colorshift;

impl Colorshift {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Colorshift {
    fn name(&self) -> &str {
        "colorshift"
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

        let stops = vec![
            Color::rgb(0xe9, 0x60, 0x77),
            Color::rgb(0xe6, 0x1f, 0x44),
            Color::rgb(0x8c, 0x1b, 0x9f),
            Color::rgb(0x49, 0x1c, 0x8c),
            Color::rgb(0x20, 0x5c, 0xaa),
            Color::rgb(0x0e, 0xaa, 0x94),
            Color::rgb(0x16, 0xdb, 0x75),
            Color::rgb(0x8c, 0xe0, 0x1e),
        ];
        let gradient = Gradient::new(stops, 36);
        let palette = gradient.colors();
        let pal_len = palette.len().max(1);

        let ids: Vec<_> = term.get_characters().iter().map(|c| c.id).collect();
        for id in &ids {
            term.set_character_visibility(*id, true);
        }

        let cycles = pal_len * 2;
        let mut out = Vec::with_capacity(cycles);

        for frame_i in 0..cycles {
            let chars_snapshot: Vec<_> = term
                .get_characters()
                .iter()
                .map(|c| {
                    (
                        c.id,
                        c.input_symbol,
                        c.input_coord.column,
                        c.input_coord.row,
                    )
                })
                .collect();

            for (id, symbol, col, row) in &chars_snapshot {
                let idx = ((*col as usize)
                    .wrapping_add(*row as usize)
                    .wrapping_add(frame_i))
                    % pal_len;
                let color = palette[idx];
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    ch.colors = Some(ColorPair {
                        fg: Some(color),
                        bg: None,
                    });
                    ch.animation.set_appearance(*symbol, Some(color));
                    let scn_id = format!("cs-{}-{}", id.0, frame_i);
                    {
                        let scn = ch.animation.new_scene(scn_id.clone());
                        scn.add_frame(
                            *symbol,
                            1,
                            Some(ColorPair {
                                fg: Some(color),
                                bg: None,
                            }),
                        );
                    }
                    ch.animation.activate_scene(&scn_id);
                }
            }

            term.step_all();
            let raw = term.get_formatted_output_string();
            let mut painted = String::new();
            for ch in term.get_characters() {
                if !ch.is_visible {
                    continue;
                }
                let color = ch
                    .animation
                    .current_character_visual
                    .colors
                    .and_then(|p| p.fg)
                    .or_else(|| ch.colors.and_then(|p| p.fg))
                    .unwrap_or(Color::rgb(255, 255, 255));
                painted.push_str(&format!(
                    "\x1b[{};{}H\x1b[38;2;{};{};{}m{}",
                    ch.current_coord.row + 1,
                    ch.current_coord.column + 1,
                    color.r,
                    color.g,
                    color.b,
                    ch.animation.current_character_visual.symbol
                ));
            }
            painted.push_str("\x1b[0m");
            if painted.chars().filter(|c| !c.is_control()).count() == 0 {
                for line in raw.lines() {
                    painted.push_str("\x1b[38;2;233;96;119m");
                    painted.push_str(line);
                    painted.push_str("\x1b[0m\n");
                }
            }
            out.push(painted);
        }
        out
    }
}
