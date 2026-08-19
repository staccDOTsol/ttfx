use crate::utils::geometry::Coord;

/// A waypoint in a motion path.
#[derive(Clone, Debug)]
pub struct Waypoint {
    pub id: String,
    pub coord: Coord,
    pub ease: Option<String>,
}

impl Waypoint {
    pub fn new(id: &str, coord: Coord) -> Self {
        Waypoint {
            id: id.to_string(),
            coord,
            ease: None,
        }
    }
}

/// A segment connecting two waypoints.
#[derive(Clone, Debug)]
pub struct Segment {
    pub start: Waypoint,
    pub end: Waypoint,
}

impl Segment {
    pub fn new(start: Waypoint, end: Waypoint) -> Self {
        Segment { start, end }
    }
}

/// A path composed of segments.
#[derive(Clone, Debug)]
pub struct Path {
    pub id: String,
    pub segments: Vec<Segment>,
}

impl Path {
    pub fn new(id: &str) -> Self {
        Path {
            id: id.to_string(),
            segments: Vec::new(),
        }
    }

    pub fn add_segment(&mut self, segment: Segment) {
        self.segments.push(segment);
    }
}

/// Motion controller for a character.
pub struct Motion {
    pub paths: Vec<Path>,
    pub active_path: Option<usize>,
    pub current_segment: usize,
    pub current_position: Coord,
}

impl Motion {
    pub fn new(start: Coord) -> Self {
        Motion {
            paths: Vec::new(),
            active_path: None,
            current_segment: 0,
            current_position: start,
        }
    }

    pub fn add_path(&mut self, path: Path) {
        self.paths.push(path);
        if self.active_path.is_none() {
            self.active_path = Some(self.paths.len() - 1);
        }
    }

    pub fn step(&mut self) -> Option<Coord> {
        if let Some(path_idx) = self.active_path {
            let path = &self.paths[path_idx];
            if self.current_segment < path.segments.len() {
                let segment = &path.segments[self.current_segment];
                // Simple linear interpolation for now
                let start = segment.start.coord;
                let end = segment.end.coord;
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let steps = dx.abs().max(dy.abs()).max(1);
                let step_x = dx as f32 / steps as f32;
                let step_y = dy as f32 / steps as f32;
                // Move one step
                let new_x = self.current_position.x as f32 + step_x;
                let new_y = self.current_position.y as f32 + step_y;
                self.current_position = Coord::new(new_x.round() as i32, new_y.round() as i32);
                if self.current_position == end {
                    self.current_segment += 1;
                }
                Some(self.current_position)
            } else {
                None
            }
        } else {
            None
        }
    }
}
