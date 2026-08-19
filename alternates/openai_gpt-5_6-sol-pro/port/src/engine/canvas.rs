use crate::engine::character::EffectCharacter;
use crate::utils::geometry::Coord;
use crate::utils::graphics::Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub symbol: String,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: " ".to_owned(),
            style: Style::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        Self {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn contains(&self, coord: Coord) -> bool {
        coord.column >= 0
            && coord.row >= 0
            && (coord.column as usize) < self.width
            && (coord.row as usize) < self.height
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn fill(&mut self, symbol: impl Into<String>, style: Style) {
        let symbol = symbol.into();
        for cell in &mut self.cells {
            cell.symbol.clone_from(&symbol);
            cell.style = style;
        }
    }

    pub fn get(&self, coord: Coord) -> Option<&Cell> {
        self.index(coord).map(|index| &self.cells[index])
    }

    pub fn get_mut(&mut self, coord: Coord) -> Option<&mut Cell> {
        self.index(coord).map(|index| &mut self.cells[index])
    }

    pub fn set(
        &mut self,
        coord: Coord,
        symbol: impl Into<String>,
        style: Style,
    ) -> bool {
        let Some(index) = self.index(coord) else {
            return false;
        };

        self.cells[index] = Cell {
            symbol: symbol.into(),
            style,
        };
        true
    }

    pub fn draw_character(&mut self, character: &EffectCharacter) -> bool {
        if !character.visible {
            return false;
        }

        self.set(
            character.position,
            character.symbol.clone(),
            character.style,
        )
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        let mut active_style = Style::default();

        for row in 0..self.height {
            for column in 0..self.width {
                let cell = &self.cells[row * self.width + column];

                if cell.style != active_style {
                    output.push_str("\x1b[0m");
                    output.push_str(&cell.style.ansi_prefix());
                    active_style = cell.style;
                }

                output.push_str(&cell.symbol);
            }

            if row + 1 < self.height {
                output.push('\n');
            }
        }

        if active_style != Style::default() {
            output.push_str("\x1b[0m");
        }

        output
    }

    fn index(&self, coord: Coord) -> Option<usize> {
        if !self.contains(coord) {
            return None;
        }

        Some(coord.row as usize * self.width + coord.column as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::graphics::Color;

    #[test]
    fn sets_and_renders_cells() {
        let mut canvas = Canvas::new(2, 1);
        canvas.set(
            Coord::new(0, 0),
            "A",
            Style {
                foreground: Some(Color::new(255, 0, 0)),
                ..Style::default()
            },
        );
        canvas.set(Coord::new(1, 0), "B", Style::default());

        let rendered = canvas.render();
        assert!(rendered.contains('A'));
        assert!(rendered.contains('B'));
    }
}
