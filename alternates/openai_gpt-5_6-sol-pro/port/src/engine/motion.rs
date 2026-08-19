use std::collections::BTreeMap;

use crate::utils::geometry::Coord;

#[derive(Clone, Debug, PartialEq, Eq)]
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
pub struct Path {
    pub id: String,
    pub speed: f64,
    pub loop_path: bool,
    waypoints: Vec<Waypoint>,
}

impl Path {
    pub fn new(id: impl Into<String>, speed: f64) -> Self {
        Self {
            id: id.into(),
            speed: speed.max(0.0),
            loop_path: false,
            waypoints: Vec::new(),
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    pub fn waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }

    pub fn with_looping(mut self, loop_path: bool) -> Self {
        self.loop_path = loop_path;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct Motion {
    paths: BTreeMap<String, Path>,
    active_path: Option<String>,
    waypoint_index: usize,
    precise_position: Option<(f64, f64)>,
}

impl Motion {
    pub fn add_path(&mut self, path: Path) -> Option<Path> {
        self.paths.insert(path.id.clone(), path)
    }

    pub fn path(&self, id: &str) -> Option<&Path> {
        self.paths.get(id)
    }

    pub fn path_mut(&mut self, id: &str) -> Option<&mut Path> {
        self.paths.get_mut(id)
    }

    pub fn activate(&mut self, id: &str, current_position: Coord) -> bool {
        let Some(path) = self.paths.get(id) else {
            return false;
        };

        if path.waypoints.is_empty() || path.speed <= 0.0 {
            return false;
        }

        self.active_path = Some(id.to_owned());
        self.waypoint_index = 0;
        self.precise_position = Some((
            current_position.column as f64,
            current_position.row as f64,
        ));
        true
    }

    pub fn deactivate(&mut self) {
        self.active_path = None;
        self.precise_position = None;
        self.waypoint_index = 0;
    }

    pub fn is_active(&self) -> bool {
        self.active_path.is_some()
    }

    pub fn step(&mut self, current_position: Coord) -> Option<Coord> {
        let path_id = self.active_path.clone()?;
        let path = self.paths.get(&path_id)?;
        let speed = path.speed;
        let loop_path = path.loop_path;
        let waypoints = path.waypoints.clone();

        if waypoints.is_empty() || speed <= 0.0 {
            self.deactivate();
            return None;
        }

        let (mut x, mut y) = self.precise_position.unwrap_or((
            current_position.column as f64,
            current_position.row as f64,
        ));
        let mut remaining = speed;

        while remaining > 0.0 {
            if self.waypoint_index >= waypoints.len() {
                if loop_path {
                    self.waypoint_index = 0;
                } else {
                    self.deactivate();
                    return Some(Coord::new(x.round() as i32, y.round() as i32));
                }
            }

            let target = waypoints[self.waypoint_index].coord;
            let dx = target.column as f64 - x;
            let dy = target.row as f64 - y;
            let distance = dx.hypot(dy);

            if distance <= f64::EPSILON {
                self.waypoint_index += 1;
                continue;
            }

            if remaining >= distance {
                x = target.column as f64;
                y = target.row as f64;
                remaining -= distance;
                self.waypoint_index += 1;

                if self.waypoint_index >= waypoints.len() && !loop_path {
                    self.active_path = None;
                    self.precise_position = Some((x, y));
                    return Some(target);
                }
            } else {
                let ratio = remaining / distance;
                x += dx * ratio;
                y += dy * ratio;
                remaining = 0.0;
            }
        }

        self.precise_position = Some((x, y));
        Some(Coord::new(x.round() as i32, y.round() as i32))
    }
}
