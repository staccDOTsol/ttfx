//! Coordinates and geometric helpers (mirrors terminaltexteffects.utils.geometry).

/// A coordinate on the canvas. Column 1, row 1 is the bottom-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Coord {
    pub column: i32,
    pub row: i32,
}

impl Coord {
    pub fn new(column: i32, row: i32) -> Self {
        Self { column, row }
    }
}

/// Euclidean length of the line between two coordinates.
pub fn find_length_of_line(a: Coord, b: Coord) -> f64 {
    let dx = (b.column - a.column) as f64;
    let dy = (b.row - a.row) as f64;
    (dx * dx + dy * dy).sqrt()
}

/// The coordinate found by linearly interpolating `t` (0..=1) of the way
/// from `start` to `end`, rounded to the nearest cell.
pub fn lerp_coord(start: Coord, end: Coord, t: f64) -> Coord {
    let column = start.column as f64 + (end.column - start.column) as f64 * t;
    let row = start.row as f64 + (end.row - start.row) as f64 * t;
    Coord::new(column.round() as i32, row.round() as i32)
}

/// Coordinates approximating a circle of `radius` around `origin`.
pub fn find_coords_on_circle(origin: Coord, radius: i32, points: usize) -> Vec<Coord> {
    let mut coords = Vec::with_capacity(points);
    if points == 0 {
        return coords;
    }
    for i in 0..points {
        let angle = (i as f64 / points as f64) * std::f64::consts::TAU;
        let column = origin.column as f64 + angle.cos() * radius as f64;
        let row = origin.row as f64 + angle.sin() * radius as f64;
        coords.push(Coord::new(column.round() as i32, row.round() as i32));
    }
    coords
}
