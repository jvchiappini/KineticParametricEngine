use glam::DVec2;
use serde::{Deserialize, Serialize};

pub type EntityId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub id: EntityId,
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn pos(&self) -> DVec2 {
        DVec2::new(self.x, self.y)
    }

    pub fn set_pos(&mut self, p: DVec2) {
        self.x = p.x;
        self.y = p.y;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub id: EntityId,
    pub start: EntityId,
    pub end: EntityId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arc {
    pub id: EntityId,
    pub center: EntityId,
    pub start: EntityId,
    pub end: EntityId,
    pub radius: f64,
    pub sweep_angle: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub id: EntityId,
    pub center: EntityId,
    pub radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeometryType {
    Point(Point),
    Line(Line),
    Arc(Arc),
    Circle(Circle),
}

pub fn midpoint(a: DVec2, b: DVec2) -> DVec2 {
    (a + b) * 0.5
}

pub fn distance(a: DVec2, b: DVec2) -> f64 {
    a.distance(b)
}

pub fn angle_between(a: DVec2, b: DVec2, c: DVec2) -> f64 {
    let ba = a - b;
    let bc = c - b;
    ba.angle_to(bc)
}

pub fn point_on_line(p: DVec2, a: DVec2, b: DVec2, tolerance: f64) -> bool {
    let d = distance(a, b);
    if d < 1e-12 {
        return distance(p, a) < tolerance;
    }
    let t = ((p - a).dot(b - a)) / (d * d);
    if t < 0.0 || t > 1.0 {
        return false;
    }
    let closest = a + (b - a) * t;
    distance(p, closest) < tolerance
}

pub fn closest_point_on_line(p: DVec2, a: DVec2, b: DVec2) -> DVec2 {
    let d = b - a;
    let len2 = d.length_squared();
    if len2 < 1e-12 {
        return a;
    }
    let t = ((p - a).dot(d) / len2).clamp(0.0, 1.0);
    a + d * t
}
