//! Waypoints, segments, paths, and per-character motion state.

use std::collections::HashMap;

use crate::utils::easing::EasingFunction;
use crate::utils::geometry::{find_length_of_line, lerp_coord, Coord};

/// A named target coordinate along a path.
#[derive(Debug, Clone)]
pub struct Waypoint {
    pub waypoint_id: String,
    pub coord: Coord,
}

/// A straight segment between two coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub start: Coord,
    pub end: Coord,
    pub distance: f64,
}

impl Segment {
    pub fn new(start: Coord, end: Coord) -> Self {
        Self {
            start,
            end,
            distance: find_length_of_line(start, end),
        }
    }

    /// Coordinate at `distance` along the segment.
    pub fn coord_at_distance(&self, distance: f64) -> Coord {
        if self.distance == 0.0 {
            return self.end;
        }
        lerp_coord(self.start, self.end, (distance / self.distance).clamp(0.0, 1.0))
    }
}

/// A path: an ordered list of waypoints traversed at `speed` cells per tick.
#[derive(Debug, Clone)]
pub struct Path {
    pub path_id: String,
    pub speed: f64,
    pub ease: Option<EasingFunction>,
    pub waypoints: Vec<Waypoint>,
    pub segments: Vec<Segment>,
    pub origin: Option<Coord>,
    pub total_distance: f64,
    pub current_step: u32,
    pub max_steps: u32,
}

impl Path {
    pub fn new(path_id: &str, speed: f64, ease: Option<EasingFunction>) -> Self {
        Self {
            path_id: path_id.to_string(),
            speed: if speed > 0.0 { speed } else { 1.0 },
            ease,
            waypoints: Vec::new(),
            segments: Vec::new(),
            origin: None,
            total_distance: 0.0,
            current_step: 0,
            max_steps: 0,
        }
    }

    pub fn new_waypoint(&mut self, waypoint_id: &str, coord: Coord) {
        self.waypoints.push(Waypoint {
            waypoint_id: waypoint_id.to_string(),
            coord,
        });
    }

    /// Prepare the path for traversal starting from `origin`.
    pub fn activate(&mut self, origin: Coord) {
        self.origin = Some(origin);
        self.segments.clear();
        let mut prev = origin;
        for waypoint in &self.waypoints {
            self.segments.push(Segment::new(prev, waypoint.coord));
            prev = waypoint.coord;
        }
        self.total_distance = self.segments.iter().map(|s| s.distance).sum();
        self.current_step = 0;
        self.max_steps = ((self.total_distance / self.speed).ceil() as u32).max(1);
    }

    pub fn is_complete(&self) -> bool {
        self.max_steps > 0 && self.current_step >= self.max_steps
    }

    fn coord_at_distance(&self, distance: f64) -> Coord {
        let mut remaining = distance.clamp(0.0, self.total_distance);
        for segment in &self.segments {
            if remaining <= segment.distance {
                return segment.coord_at_distance(remaining);
            }
            remaining -= segment.distance;
        }
        self.segments
            .last()
            .map(|s| s.end)
            .or(self.origin)
            .unwrap_or_default()
    }

    /// Advance one step along the path and return the new coordinate.
    pub fn step(&mut self) -> Coord {
        if self.max_steps == 0 {
            return self.origin.unwrap_or_default();
        }
        if self.current_step < self.max_steps {
            self.current_step += 1;
        }
        let t = self.current_step as f64 / self.max_steps as f64;
        let eased = self.ease.map(|f| f(t)).unwrap_or(t);
        self.coord_at_distance(eased * self.total_distance)
    }
}

/// Motion state for a single character.
#[derive(Debug, Clone)]
pub struct Motion {
    pub current_coord: Coord,
    pub paths: HashMap<String, Path>,
    pub active_path_id: Option<String>,
}

impl Motion {
    pub fn new(start: Coord) -> Self {
        Self {
            current_coord: start,
            paths: HashMap::new(),
            active_path_id: None,
        }
    }

    pub fn new_path(&mut self, path_id: &str, speed: f64, ease: Option<EasingFunction>) -> &mut Path {
        self.paths
            .entry(path_id.to_string())
            .or_insert_with(|| Path::new(path_id, speed, ease))
    }

    pub fn query_path(&mut self, path_id: &str) -> Option<&mut Path> {
        self.paths.get_mut(path_id)
    }

    pub fn activate_path(&mut self, path_id: &str) {
        let origin = self.current_coord;
        if let Some(path) = self.paths.get_mut(path_id) {
            path.activate(origin);
            self.active_path_id = Some(path_id.to_string());
        }
    }

    pub fn movement_is_complete(&self) -> bool {
        self.active_path_id.is_none()
    }

    /// Advance the active path by one step.
    pub fn move_(&mut self) {
        if let Some(id) = self.active_path_id.clone() {
            let mut finished = false;
            if let Some(path) = self.paths.get_mut(&id) {
                self.current_coord = path.step();
                finished = path.is_complete();
            }
            if finished {
                self.active_path_id = None;
            }
        }
    }
}
