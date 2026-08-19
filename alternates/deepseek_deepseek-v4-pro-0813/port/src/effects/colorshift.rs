use super::Effect;
use crate::utils::graphics::{Color, Gradient};

pub struct Colorshift;

impl Colorshift {
    pub fn new() -> Self {
        Colorshift
    }
}

impl Effect for Colorshift {
    fn name(&self) -> &str {
        "colorshift"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        // Build a rainbow gradient representing the color spectrum the original
        // effect moves through. Existing engine gradient facilities are used to
        // produce RGB colors for each character.
        let gradient = Gradient::new(vec![
            (0.0, Color::RED),
            (1.0 / 6.0, Color::new(255, 165, 0)),   // orange
            (2.0 / 6.0, Color::new(255, 255, 0)),   // yellow
            (3.0 / 6.0, Color::GREEN),
            (4.0 / 6.0, Color::BLUE),
            (5.0 / 6.0, Color::new(75, 0, 130)),    // indigo
            (1.0, Color::new(238, 130, 238)),       // violet
        ]);

        let total_frames: usize = 60;
        let mut frames = Vec::with_capacity(total_frames);

        let lines: Vec<&str> = input.split_terminator('\n').collect();

        for frame_idx in 0..total_frames {
            let animation_t = frame_idx as f64 / total_frames as f64;
            let mut output = String::new();

            for (row_idx, line) in lines.iter().enumerate() {
                for (col_idx, ch) in line.chars().enumerate() {
                    // Shift hue by position and animation frame so each character
                    // cycles through the gradient at its own phase.
                    let position_offset = (row_idx + col_idx) as f64 * 0.05;
                    let t = (animation_t + position_offset) % 1.0;
                    let color = gradient.color_at(t);

                    output.push_str(&format!(
                        "\x1b[38;2;{};{};{}m{}\x1b[0m",
                        color.r, color.g, color.b, ch
                    ));
                }

                if row_idx < lines.len() - 1 {
                    output.push('\n');
                }
            }

            // Ensure the terminal starts and ends in a reset state.
            output.push_str("\x1b[0m");
            frames.push(output);
        }

        frames
    }
}
