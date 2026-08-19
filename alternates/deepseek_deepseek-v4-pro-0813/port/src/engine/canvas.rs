use crate::engine::character::EffectCharacter;
use crate::utils::geometry::Coord;
use crate::utils::graphics::{Color, ColorPair};

/// A single cell in the canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub style: CellStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CellStyle {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strike: bool,
}

impl Default for CellStyle {
    fn default() -> Self {
        CellStyle {
            fg: Color::default(),
            bg: Color::default(),
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strike: false,
        }
    }
}

/// A 2D grid of cells representing the terminal screen.
pub struct Canvas {
    pub width: u16,
    pub height: u16,
    cells: Vec<Cell>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![
            Cell {
                symbol: ' ',
                style: CellStyle::default(),
            };
            (width as usize) * (height as usize)
        ];
        Canvas {
            width,
            height,
            cells,
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.symbol = ' ';
            cell.style = CellStyle::default();
        }
    }

    pub fn set_cell(&mut self, coord: Coord, symbol: char, style: CellStyle) {
        if coord.x >= 0 && coord.x < self.width as i32 && coord.y >= 0 && coord.y < self.height as i32 {
            let idx = (coord.y as usize) * (self.width as usize) + (coord.x as usize);
            self.cells[idx] = Cell { symbol, style };
        }
    }

    pub fn get_cell(&self, coord: Coord) -> Option<&Cell> {
        if coord.x >= 0 && coord.x < self.width as i32 && coord.y >= 0 && coord.y < self.height as i32 {
            let idx = (coord.y as usize) * (self.width as usize) + (coord.x as usize);
            Some(&self.cells[idx])
        } else {
            None
        }
    }

    /// Render the canvas to a string using ANSI escape codes.
    pub fn render(&self, characters: &[EffectCharacter]) -> String {
        // First, clear the canvas and draw characters
        let mut temp_canvas = self.clone();
        temp_canvas.clear();

        for character in characters {
            if character.visible {
                let style = CellStyle {
                    fg: character.color_pair.fg,
                    bg: character.color_pair.bg,
                    bold: character.bold,
                    dim: character.dim,
                    italic: character.italic,
                    underline: character.underline,
                    blink: character.blink,
                    reverse: character.reverse,
                    hidden: character.hidden,
                    strike: character.strike,
                };
                temp_canvas.set_cell(character.position, character.symbol, style);
            }
        }

        // Build ANSI string
        let mut output = String::new();
        let mut last_style: Option<CellStyle> = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = temp_canvas.get_cell(Coord::new(x as i32, y as i32)).unwrap();
                let style = &cell.style;
                if Some(style) != last_style.as_ref() {
                    output.push_str(&style_to_ansi(style));
                    last_style = Some(style.clone());
                }
                output.push(cell.symbol);
            }
            output.push('\n');
        }
        // Reset style at end
        output.push_str("\x1b[0m");
        output
    }
}

impl Clone for Canvas {
    fn clone(&self) -> Self {
        Canvas {
            width: self.width,
            height: self.height,
            cells: self.cells.clone(),
        }
    }
}

fn style_to_ansi(style: &CellStyle) -> String {
    let mut codes = Vec::new();
    if style.bold {
        codes.push("1".to_string());
    }
    if style.dim {
        codes.push("2".to_string());
    }
    if style.italic {
        codes.push("3".to_string());
    }
    if style.underline {
        codes.push("4".to_string());
    }
    if style.blink {
        codes.push("5".to_string());
    }
    if style.reverse {
        codes.push("7".to_string());
    }
    if style.hidden {
        codes.push("8".to_string());
    }
    if style.strike {
        codes.push("9".to_string());
    }
    // Foreground color
    codes.push(format!("38;2;{};{};{}", style.fg.r, style.fg.g, style.fg.b));
    // Background color
    codes.push(format!("48;2;{};{};{}", style.bg.r, style.bg.g, style.bg.b));

    format!("\x1b[{}m", codes.join(";"))
}
