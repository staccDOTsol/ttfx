//! The Canvas: a grid of styled cells with bottom-left origin (1,1).

use crate::engine::animation::CharacterVisual;
use crate::utils::geometry::Coord;

/// A rectangular grid of optional styled cells.
#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: i32,
    pub height: i32,
    cells: Vec<Option<CharacterVisual>>,
}

impl Canvas {
    pub fn new(width: i32, height: i32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self {
            width,
            height,
            cells: vec![None; (width * height) as usize],
        }
    }

    /// Coordinate of the canvas center cell.
    pub fn center(&self) -> Coord {
        Coord::new((self.width + 1) / 2, (self.height + 1) / 2)
    }

    pub fn coord_is_in_canvas(&self, coord: Coord) -> bool {
        coord.column >= 1 && coord.column <= self.width && coord.row >= 1 && coord.row <= self.height
    }

    fn index(&self, coord: Coord) -> Option<usize> {
        if !self.coord_is_in_canvas(coord) {
            return None;
        }
        // Row 1 is the bottom row; store top row first for rendering.
        let row_from_top = (self.height - coord.row) as usize;
        let col = (coord.column - 1) as usize;
        Some(row_from_top * self.width as usize + col)
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = None;
        }
    }

    pub fn set_cell(&mut self, coord: Coord, visual: CharacterVisual) {
        if let Some(idx) = self.index(coord) {
            self.cells[idx] = Some(visual);
        }
    }

    pub fn get_cell(&self, coord: Coord) -> Option<&CharacterVisual> {
        self.index(coord).and_then(|idx| self.cells[idx].as_ref())
    }

    /// Render the canvas to a newline-joined frame string.
    pub fn to_frame_string(&self) -> String {
        let mut out = String::with_capacity((self.width as usize + 1) * self.height as usize);
        for row in 0..self.height as usize {
            for col in 0..self.width as usize {
                match &self.cells[row * self.width as usize + col] {
                    Some(visual) => out.push_str(&visual.formatted()),
                    None => out.push(' '),
                }
            }
            if row + 1 < self.height as usize {
                out.push('\n');
            }
        }
        out
    }
}
