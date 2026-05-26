use kpe_schema::geometry::{Sketch2D, SketchDef, SketchPrimitive};
use glam::DVec2;

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
                if points.len() >= 3 {
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
