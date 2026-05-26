use glam::DVec2;
use serde::{Serialize, Deserialize};
use crate::sketch::entities::*;
use crate::sketch::constraints::Constraint;

const SNAP_DIST: f64 = 0.15;
const INFERENCE_DIST: f64 = 0.5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapResult {
    pub x: f64,
    pub y: f64,
    pub kind: String,
    pub target_id: Option<EntityId>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapKind {
    Grid,
    Endpoint,
    Midpoint,
    OnEntity,
    Intersection,
    Horizontal,
    Vertical,
    Extension,
}

pub struct InferenceEngine;

impl InferenceEngine {
    pub fn snap_to_grid_point(pos: DVec2, grid_size: f64) -> DVec2 {
        let x = (pos.x / grid_size).round() * grid_size;
        let y = (pos.y / grid_size).round() * grid_size;
        DVec2::new(x, y)
    }

    pub fn snap_to_entities(
        pos: DVec2,
        points: &[Point],
        lines: &[Line],
        _arcs: &[Arc],
        circles: &[Circle],
    ) -> Option<SnapResult> {
        if let Some(result) = Self::snap_endpoint(pos, points) {
            return Some(result);
        }

        if let Some(result) = Self::snap_midpoint(pos, points, lines) {
            return Some(result);
        }

        if let Some(result) = Self::snap_on_line(pos, lines, points) {
            return Some(result);
        }

        for c in circles {
            let center = point_by_id(points, c.center);
            let d = pos.distance(center);
            if (d - c.radius).abs() < SNAP_DIST {
                return Some(SnapResult {
                    x: pos.x,
                    y: pos.y,
                    kind: "on_entity".into(),
                    target_id: Some(c.id),
                });
            }
        }

        None
    }

    pub fn infer_constraints(
        _points: &[Point],
        lines: &[Line],
        new_line_id: EntityId,
        new_start: DVec2,
        new_end: DVec2,
    ) -> Vec<Constraint> {
        let mut inferred = Vec::new();

        let dx = (new_end.x - new_start.x).abs();
        let dy = (new_end.y - new_start.y).abs();

        if dy < INFERENCE_DIST && dx > INFERENCE_DIST * 2.0 {
            inferred.push(Constraint::Horizontal { line: new_line_id });
        }

        if dx < INFERENCE_DIST && dy > INFERENCE_DIST * 2.0 {
            inferred.push(Constraint::Vertical { line: new_line_id });
        }

        for existing in lines {
            if existing.id == new_line_id {
                continue;
            }
            let a1 = point_by_id(_points, existing.start);
            let a2 = point_by_id(_points, existing.end);
            let dir_existing = (a2 - a1).normalize();
            let dir_new = (new_end - new_start).normalize();

            if dir_existing.perp_dot(dir_new).abs() < 0.05 {
                inferred.push(Constraint::Parallel { line_a: existing.id, line_b: new_line_id });
                break;
            }

            if dir_existing.dot(dir_new).abs() < 0.05 {
                inferred.push(Constraint::Perpendicular { line_a: existing.id, line_b: new_line_id });
                break;
            }
        }

        inferred
    }

    fn snap_endpoint(pos: DVec2, points: &[Point]) -> Option<SnapResult> {
        for pt in points {
            let d = pos.distance(pt.pos());
            if d < SNAP_DIST {
                return Some(SnapResult {
                    x: pt.x,
                    y: pt.y,
                    kind: "endpoint".into(),
                    target_id: Some(pt.id),
                });
            }
        }
        None
    }

    fn snap_midpoint(pos: DVec2, points: &[Point], lines: &[Line]) -> Option<SnapResult> {
        for line in lines {
            let a = point_by_id(points, line.start);
            let b = point_by_id(points, line.end);
            let mid = (a + b) * 0.5;
            if pos.distance(mid) < SNAP_DIST * 0.8 {
                return Some(SnapResult {
                    x: mid.x,
                    y: mid.y,
                    kind: "midpoint".into(),
                    target_id: Some(line.id),
                });
            }
        }
        None
    }

    fn snap_on_line(pos: DVec2, lines: &[Line], points: &[Point]) -> Option<SnapResult> {
        for line in lines {
            let a = point_by_id(points, line.start);
            let b = point_by_id(points, line.end);
            let closest = closest_point_on_line(pos, a, b);
            if pos.distance(closest) < SNAP_DIST && closest.distance(a) > 0.01 && closest.distance(b) > 0.01 {
                return Some(SnapResult {
                    x: closest.x,
                    y: closest.y,
                    kind: "on_entity".into(),
                    target_id: Some(line.id),
                });
            }
        }
        None
    }
}

fn point_by_id(points: &[Point], id: EntityId) -> DVec2 {
    points.iter().find(|p| p.id == id).map(|p| p.pos()).unwrap_or(DVec2::ZERO)
}
