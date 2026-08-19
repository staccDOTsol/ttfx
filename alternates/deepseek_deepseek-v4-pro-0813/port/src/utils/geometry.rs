/// A 2D coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Coord { x, y }
    }

    pub fn distance(&self, other: &Coord) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Compute the length of a line between two points.
pub fn find_length_of_line(start: Coord, end: Coord) -> f64 {
    start.distance(&end)
}

/// Compute the length of a Bezier curve (simplified: straight line approximation).
pub fn find_length_of_bezier_curve(
    start: Coord,
    control1: Coord,
    control2: Coord,
    end: Coord,
) -> f64 {
    // For now, approximate as sum of line segments
    let mut length = 0.0;
    let steps = 20;
    let mut prev = start;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = (1.0 - t).powi(3) * start.x as f64
            + 3.0 * (1.0 - t).powi(2) * t * control1.x as f64
            + 3.0 * (1.0 - t) * t.powi(2) * control2.x as f64
            + t.powi(3) * end.x as f64;
        let y = (1.0 - t).powi(3) * start.y as f64
            + 3.0 * (1.0 - t).powi(2) * t * control1.y as f64
            + 3.0 * (1.0 - t) * t.powi(2) * control2.y as f64
            + t.powi(3) * end.y as f64;
        let current = Coord::new(x.round() as i32, y.round() as i32);
        length += find_length_of_line(prev, current);
        prev = current;
    }
    length
}
