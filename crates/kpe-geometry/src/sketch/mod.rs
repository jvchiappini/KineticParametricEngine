pub mod entities;
pub mod constraints;
pub mod solver;
pub mod inference;
pub mod boolean;
pub mod document;
pub mod spatial;

use kpe_schema::geometry::{Sketch2D, SketchDef, SketchPrimitive};
use glam::DVec2;

pub use entities::*;
pub use constraints::Constraint;
pub use solver::{Solver, analyze_dof};
pub use inference::{InferenceEngine, SnapResult};
pub use boolean::{boolean_contours, BooleanOp, extrude_contour_to_3d};
pub use document::SketchDocument;

pub struct SketchEngine;

impl SketchEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn rectangle(&self, width: f64, height: f64) -> Sketch2D {
        let half_w = width / 2.0;
        let half_h = height / 2.0;
        Sketch2D {
            contours: vec![vec![
                [-half_w, -half_h],
                [half_w, -half_h],
                [half_w, half_h],
                [-half_w, half_h],
                [-half_w, -half_h],
            ]],
        }
    }

    pub fn circle(&self, radius: f64, segments: u32) -> Sketch2D {
        let mut contour = Vec::new();
        for i in 0..=segments {
            let angle = (i as f64 / segments as f64) * std::f64::consts::TAU;
            contour.push([radius * angle.cos(), radius * angle.sin()]);
        }
        Sketch2D {
            contours: vec![contour],
        }
    }

    pub fn extrude(&self, sketch: &Sketch2D, _height: f64) -> Sketch2D {
        sketch.clone()
    }

    pub fn revolve(&self, sketch: &Sketch2D, _angle: f64) -> Sketch2D {
        sketch.clone()
    }
}

impl Default for SketchEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Ear-clipping triangulation for a simple polygon.
///
/// The contour must be a closed simple polygon (last point != first).
/// Returns triangle indices into the original points list.
pub fn triangulate_contour(contour: &[DVec2]) -> Vec<[u32; 3]> {
    let n = contour.len();
    if n < 3 {
        return vec![];
    }

    let mut remaining: Vec<usize> = (0..n).collect();
    let mut triangles = Vec::new();

    while remaining.len() > 3 {
        let len = remaining.len();
        let mut ear_found = false;

        for i in 0..len {
            let prev = remaining[(i + len - 1) % len];
            let curr = remaining[i];
            let next = remaining[(i + 1) % len];

            let a = contour[prev];
            let b = contour[curr];
            let c = contour[next];

            let cross = (b - a).perp_dot(c - b);
            if cross <= 0.0 {
                continue;
            }

            let mut is_ear = true;
            for &j in &remaining {
                if j == prev || j == curr || j == next {
                    continue;
                }
                if point_in_triangle(contour[j], a, b, c) {
                    is_ear = false;
                    break;
                }
            }

            if is_ear {
                triangles.push([prev as u32, curr as u32, next as u32]);
                remaining.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            let last = remaining.len();
            if last >= 3 {
                triangles.push([remaining[0] as u32, remaining[1] as u32, remaining[2] as u32]);
            }
            break;
        }
    }

    if remaining.len() == 3 {
        triangles.push([remaining[0] as u32, remaining[1] as u32, remaining[2] as u32]);
    }

    triangles
}

fn point_in_triangle(p: DVec2, a: DVec2, b: DVec2, c: DVec2) -> bool {
    let cross1 = (b - a).perp_dot(p - a);
    let cross2 = (c - b).perp_dot(p - b);
    let cross3 = (a - c).perp_dot(p - c);
    (cross1 >= 0.0 && cross2 >= 0.0 && cross3 >= 0.0)
        || (cross1 <= 0.0 && cross2 <= 0.0 && cross3 <= 0.0)
}

/// Returns true if p is inside the axis-aligned rectangle.
pub fn point_in_rect(p: DVec2, x: f64, y: f64, w: f64, h: f64) -> bool {
    p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h
}

pub fn tessellate_sketch(sketch: &SketchDef) -> Vec<Vec<DVec2>> {
    let mut contours: Vec<Vec<DVec2>> = Vec::new();

    for prim in &sketch.primitives {
        match prim {
            SketchPrimitive::Rectangle { x, y, width, height } => {
                contours.push(vec![
                    DVec2::new(*x, *y),
                    DVec2::new(x + width, *y),
                    DVec2::new(x + width, y + height),
                    DVec2::new(*x, y + height),
                ]);
            }
            SketchPrimitive::Circle { cx, cy, radius, segments } => {
                let n = segments.unwrap_or(32).max(3);
                let mut pts = Vec::with_capacity(n as usize);
                for i in 0..n {
                    let angle = (i as f64 / n as f64) * std::f64::consts::TAU;
                    pts.push(DVec2::new(cx + radius * angle.cos(), cy + radius * angle.sin()));
                }
                contours.push(pts);
            }
            SketchPrimitive::Polygon { points } => {
                if points.len() >= 2 {
                    let pts: Vec<DVec2> = points.iter().map(|p| DVec2::new(p[0], p[1])).collect();
                    contours.push(pts);
                }
            }
            SketchPrimitive::Arc { cx, cy, radius, start_angle, end_angle, segments } => {
                let n = segments.unwrap_or(16).max(2);
                let mut pts = Vec::with_capacity(n as usize);
                for i in 0..n {
                    let t = i as f64 / (n - 1) as f64;
                    let angle = start_angle + (end_angle - start_angle) * t;
                    pts.push(DVec2::new(cx + radius * angle.cos(), cy + radius * angle.sin()));
                }
                contours.push(pts);
            }
        }
    }

    contours
}
