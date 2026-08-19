use super::Effect;
use crate::engine::character::CharacterId;
use crate::engine::terminal::Terminal;
use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair, Gradient};

pub struct Slide;

impl Slide {
    pub fn new() -> Self {
        Self
    }
}

impl Effect for Slide {
    fn name(&self) -> &str {
        "slide"
    }

    fn frames(&self, input: &str) -> Vec<String> {
        let lines: Vec<&str> = input.lines().collect();
        let height = lines.len().max(1);
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1).max(1);
        let mut term = Terminal::from_input(input, width, height.max(1));

        let gradient = Gradient::new(
            vec![
                Color::from_hex("8A008A").unwrap_or(Color::rgb(138, 0, 138)),
                Color::from_hex("00D1FF").unwrap_or(Color::rgb(0, 209, 255)),
                Color::from_hex("FFFFFF").unwrap_or(Color::rgb(255, 255, 255)),
            ],
            12,
        );
        let pal = gradient.colors();

        // Group by row (slide from left), matching default Python grouping_direction=row / slide_direction=left
        let mut rows: Vec<Vec<CharacterId>> = vec![Vec::new(); height];
        {
            let chars = term.get_characters();
            for ch in chars {
                let r = ch.input_coord.row;
                if r >= 0 && (r as usize) < height {
                    rows[r as usize].push(ch.id);
                }
            }
        }

        let mut path_ids: Vec<(CharacterId, String)> = Vec::new();
        let n_groups = rows.len().max(1);
        for (gi, group) in rows.iter().enumerate() {
            let color = pal[gi % pal.len()];
            for id in group {
                if let Some(ch) = term.get_characters_mut().iter_mut().find(|c| c.id == *id) {
                    let start = Coord::new(-1, ch.input_coord.row);
                    ch.motion.current_coord = start;
                    ch.current_coord = start;
                    let pid = format!("slide-{}", id.0);
                    {
                        let path = ch.motion.new_path(pid.clone());
                        path.speed = 0.35;
                        path.easing = Easing::OutQuad;
                        path.new_waypoint("home", ch.input_coord);
                    }
                    ch.motion.activate_path(&pid);
                    let scn = ch.animation.new_scene("slide");
                    scn.add_frame(
                        ch.input_symbol,
                        1,
                        Some(ColorPair {
                            fg: Some(color),
                            bg: None,
                        }),
                    );
                    ch.animation.activate_scene("slide");
                    path_ids.push((*id, pid));
                }
            }
        }

        // Stagger groups: activate later rows after earlier ones have begun
        let mut frames = Vec::new();
        let max_frames = 80 + n_groups * 4;
        let mut activated = 0usize;
        for tick in 0..max_frames {
            while activated < rows.len() && tick >= activated * 3 {
                for id in &rows[activated] {
                    term.set_character_visibility(*id, true);
                }
                activated += 1;
            }
            term.step_all();
            frames.push(render_ansi(&term));
            if activated >= rows.len() && all_home(&term) {
                frames.push(render_ansi(&term));
                break;
            }
        }
        if frames.is_empty() {
            frames.push(render_ansi(&term));
        }
        frames
    }
}

fn all_home(term: &Terminal) -> bool {
    term.get_characters()
        .iter()
        .filter(|c| c.is_visible)
        .all(|c| c.current_coord == c.input_coord)
}

fn render_ansi(term: &Terminal) -> String {
    let w = term.canvas.width;
    let h = term.canvas.height;
    let mut grid: Vec<Vec<(char, Option<Color>)>> = vec![vec![(' ', None); w]; h];
    for ch in term.get_characters() {
        if !ch.is_visible {
            continue;
        }
        let x = ch.current_coord.column;
        let y = ch.current_coord.row;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let col = ch
                .animation
                .current_character_visual
                .colors
                .and_then(|p| p.fg);
            grid[y as usize][x as usize] = (ch.animation.current_character_visual.symbol, col);
        }
    }
    let mut out = String::new();
    for row in grid {
        for (sym, col) in row {
            if let Some(c) = col {
                out.push_str(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", c.r, c.g, c.b, sym));
            } else {
                out.push(sym);
            }
        }
        out.push('\n');
    }
    out
}
