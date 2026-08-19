#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Coord {
    pub column: i32,
    pub row: i32,
}

impl Coord {
    pub fn new(column: i32, row: i32) -> Self {
        Self { column, row }
    }

    pub fn distance(self, other: Coord) -> f64 {
        let dc = (self.column - other.column) as f64;
        let dr = (self.row - other.row) as f64;
        (dc * dc + dr * dr).sqrt()
    }
}

pub fn find_length_of_line(a: Coord, b: Coord) -> f64 {
    a.distance(b)
}

pub fn find_coords_on_circle(origin: Coord, radius: i32, n: usize) -> Vec<Coord> {
    if n == 0 {
        return Vec::new();
    }
    (0..n)
        .map(|i| {
            let theta = (i as f64) * std::f64::consts::TAU / (n as f64);
            Coord {
                column: origin.column + (radius as f64 * theta.cos()).round() as i32,
                row: origin.row + (radius as f64 * theta.sin()).round() as i32,
            }
        })
        .collect()
}
