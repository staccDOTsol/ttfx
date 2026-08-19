#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Coord {
    pub column: i32,
    pub row: i32,
}

impl Coord {
    pub const fn new(column: i32, row: i32) -> Self {
        Self { column, row }
    }

    pub fn distance_to(self, other: Self) -> f64 {
        let dx = (other.column - self.column) as f64;
        let dy = (other.row - self.row) as f64;
        dx.hypot(dy)
    }

    pub fn manhattan_distance_to(self, other: Self) -> u32 {
        self.column.abs_diff(other.column) + self.row.abs_diff(other.row)
    }

    pub fn lerp(self, other: Self, progress: f64) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        let column =
            self.column as f64 + (other.column - self.column) as f64 * progress;
        let row = self.row as f64 + (other.row - self.row) as f64 * progress;

        Self::new(column.round() as i32, row.round() as i32)
    }
}

pub fn find_length_of_line(start: Coord, end: Coord) -> f64 {
    start.distance_to(end)
}

pub fn quadratic_bezier(
    start: Coord,
    control: Coord,
    end: Coord,
    progress: f64,
) -> Coord {
    let t = progress.clamp(0.0, 1.0);
    let inverse = 1.0 - t;

    let column = inverse * inverse * start.column as f64
        + 2.0 * inverse * t * control.column as f64
        + t * t * end.column as f64;
    let row = inverse * inverse * start.row as f64
        + 2.0 * inverse * t * control.row as f64
        + t * t * end.row as f64;

    Coord::new(column.round() as i32, row.round() as i32)
}

pub fn cubic_bezier(
    start: Coord,
    control_1: Coord,
    control_2: Coord,
    end: Coord,
    progress: f64,
) -> Coord {
    let t = progress.clamp(0.0, 1.0);
    let inverse = 1.0 - t;

    let column = inverse.powi(3) * start.column as f64
        + 3.0 * inverse.powi(2) * t * control_1.column as f64
        + 3.0 * inverse * t.powi(2) * control_2.column as f64
        + t.powi(3) * end.column as f64;
    let row = inverse.powi(3) * start.row as f64
        + 3.0 * inverse.powi(2) * t * control_1.row as f64
        + 3.0 * inverse * t.powi(2) * control_2.row as f64
        + t.powi(3) * end.row as f64;

    Coord::new(column.round() as i32, row.round() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_coordinates() {
        let start = Coord::new(0, 0);
        let end = Coord::new(10, 20);
        assert_eq!(start.lerp(end, 0.5), Coord::new(5, 10));
    }
}
