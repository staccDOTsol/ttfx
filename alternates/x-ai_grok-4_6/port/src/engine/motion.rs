use crate::utils::easing::Easing;
use crate::utils::geometry::Coord;

#[derive(Clone, Debug)]
pub struct Waypoint {
    pub id: String,
    pub coord: Coord,
}

impl Waypoint {
    pub fn new(id: impl Into<String>, coord: Coord) -> Self {
        Self {
            id: id.into(),
            coord,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub start: Waypoint,
    pub end: Waypoint,
    pub speed: f64,
    pub easing: Easing,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub id: String,
    pub waypoints: Vec<Waypoint>,
    pub speed: f64,
    pub easing: Easing,
    pub active: bool,
    step: f64,
}

impl Path {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            waypoints: Vec::new(),
            speed: 1.0,
            easing: Easing::Linear,
            active: false,
            step: 0.0,
        }
    }

    pub fn new_waypoint(&mut self, id: impl Into<String>, coord: Coord) -> &Waypoint {
        self.waypoints.push(Waypoint::new(id, coord));
        self.waypoints.last().unwrap()
    }

    pub fn step(&mut self, current: Coord) -> Coord {
        if self.waypoints.is_empty() {
            return current;
        }
        let target = self.waypoints.last().unwrap().coord;
        let t = (self.step + self.speed * 0.05).min(1.0);
        self.step = t;
        let e = self.easing.apply(t);
        Coord {
            column: lerp_i(current.column, target.column, e),
            row: lerp_i(current.row, target.row, e),
        }
    }
}

fn lerp_i(a: i32, b: i32, t: f64) -> i32 {
    (a as f64 + (b - a) as f64 * t).round() as i32
}

#[derive(Clone, Debug)]
pub struct Motion {
    pub current_coord: Coord,
    pub active_path: Option<String>,
    paths: Vec<Path>,
}

impl Motion {
    pub fn new(coord: Coord) -> Self {
        Self {
            current_coord: coord,
            active_path: None,
            paths: Vec::new(),
        }
    }

    pub fn new_path(&mut self, id: impl Into<String>) -> &mut Path {
        self.paths.push(Path::new(id));
        self.paths.last_mut().unwrap()
    }

    pub fn query_path(&self, id: &str) -> Option<&Path> {
        self.paths.iter().find(|p| p.id == id)
    }

    pub fn activate_path(&mut self, id: &str) {
        if let Some(p) = self.paths.iter_mut().find(|p| p.id == id) {
            p.active = true;
            p.step = 0.0;
            self.active_path = Some(id.to_string());
        }
    }

    pub fn move_character(&mut self) {
        if let Some(id) = self.active_path.clone() {
            if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
                self.current_coord = path.step(self.current_coord);
            }
        }
    }
}
