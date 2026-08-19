use crate::utils::geometry::Coord;
use crate::utils::graphics::ColorPair;

#[derive(Clone, Debug, Default)]
pub struct Cell {
    pub symbol: char,
    pub colors: Option<ColorPair>,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        let n = width.saturating_mul(height);
        Self {
            width,
            height,
            cells: vec![Cell::default(); n],
        }
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    pub fn get(&self, coord: Coord) -> Option<&Cell> {
        let x = coord.column as usize;
        let y = coord.row as usize;
        self.index(x, y).map(|i| &self.cells[i])
    }

    pub fn get_mut(&mut self, coord: Coord) -> Option<&mut Cell> {
        let x = coord.column as usize;
        let y = coord.row as usize;
        self.index(x, y).map(|i| &mut self.cells[i])
    }

    pub fn set_symbol(&mut self, coord: Coord, symbol: char) {
        if let Some(cell) = self.get_mut(coord) {
            cell.symbol = symbol;
            cell.visible = true;
        }
    }

    pub fn fill(&mut self, symbol: char) {
        for cell in &mut self.cells {
            cell.symbol = symbol;
            cell.visible = true;
        }
    }

    pub fn render(&self) -> String {
        let mut out = String::with_capacity(self.width * self.height + self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = &self.cells[y * self.width + x];
                if c.visible && c.symbol != '\0' {
                    out.push(c.symbol);
                } else {
                    out.push(' ');
                }
            }
            out.push('\n');
        }
        out
    }
}
