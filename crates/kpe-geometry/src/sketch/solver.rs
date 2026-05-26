use glam::DVec2;
use crate::sketch::entities::*;
use crate::sketch::constraints::Constraint;

const MAX_ITER: usize = 200;
const CONVERGENCE: f64 = 1e-8;
const RELAXATION: f64 = 0.3;

pub struct Solver;

impl Solver {
    pub fn solve(
        points: &mut Vec<Point>,
        lines: &[Line],
        _arcs: &[Arc],
        _circles: &[Circle],
        constraints: &[Constraint],
    ) -> Result<(), String> {
        let mut vars: Vec<(usize, bool, bool)> = Vec::new();
        for (i, pt) in points.iter().enumerate() {
            let mut fixed_x = false;
            let mut fixed_y = false;
            for c in constraints {
                if let Constraint::Fix { point, x: _, y: _ } = c {
                    if *point == pt.id {
                        fixed_x = true;
                        fixed_y = true;
                    }
                }
            }
            vars.push((i, fixed_x, fixed_y));
        }

        for c in constraints {
            if let Constraint::Fix { point, x, y } = c {
                for pt in points.iter_mut() {
                    if pt.id == *point {
                        pt.x = *x;
                        pt.y = *y;
                    }
                }
            }
        }

        for _ in 0..MAX_ITER {
            let mut max_error: f64 = 0.0;

            for c in constraints {
                let err = Self::eval(points, lines, c);
                max_error = max_error.max(err.abs());
            }

            if max_error < CONVERGENCE {
                return Ok(());
            }

        for &(idx, fx, fy) in &vars {
            if fx && fy {
                continue;
            }

            let orig = DVec2::new(points[idx].x, points[idx].y);
            let eps: f64 = 1e-6;

            for c in constraints {
                let err0 = Self::eval(points, lines, c);

                let mut grad = DVec2::ZERO;

                if !fx {
                    points[idx].x = orig.x + eps;
                    let errx = Self::eval(points, lines, c);
                    grad.x = (errx - err0) / eps;
                    points[idx].x = orig.x;
                }

                if !fy {
                    points[idx].y = orig.y + eps;
                    let erry = Self::eval(points, lines, c);
                    grad.y = (erry - err0) / eps;
                    points[idx].y = orig.y;
                }

                let grad2 = grad.length_squared();
                if grad2 > 1e-20 {
                    let step = -err0 / grad2 * RELAXATION;
                    if !fx {
                        points[idx].x += grad.x * step;
                    }
                    if !fy {
                        points[idx].y += grad.y * step;
                    }
                }
            }
        }
        }

        let mut final_err: f64 = 0.0;
        for c in constraints {
            final_err = final_err.max(Self::eval(points, lines, c).abs());
        }
        if final_err > 1.0 {
            Err(format!("Solver did not converge (error: {:.6})", final_err))
        } else {
            Ok(())
        }
    }

    fn eval(points: &[Point], lines: &[Line], c: &Constraint) -> f64 {
        match c {
            Constraint::Horizontal { line } => {
                if let Some(l) = lines.iter().find(|l| l.id == *line) {
                    let a = point_by_id(points, l.start);
                    let b = point_by_id(points, l.end);
                    (a.y - b.y).abs()
                } else { 0.0 }
            }
            Constraint::Vertical { line } => {
                if let Some(l) = lines.iter().find(|l| l.id == *line) {
                    let a = point_by_id(points, l.start);
                    let b = point_by_id(points, l.end);
                    (a.x - b.x).abs()
                } else { 0.0 }
            }
            Constraint::Coincident { point_a, point_b } => {
                let a = point_by_id(points, *point_a);
                let b = point_by_id(points, *point_b);
                a.distance(b)
            }
            Constraint::Fix { point, x, y } => {
                let p = point_by_id(points, *point);
                distance(p, DVec2::new(*x, *y))
            }
            Constraint::Distance { point_a, point_b, distance } => {
                let a = point_by_id(points, *point_a);
                let b = point_by_id(points, *point_b);
                a.distance(b) - distance
            }
            Constraint::EqualLength { line_a, line_b } => {
                let la = line_by_id(lines, *line_a);
                let lb = line_by_id(lines, *line_b);
                if let (Some(la), Some(lb)) = (la, lb) {
                    let a1 = point_by_id(points, la.start);
                    let a2 = point_by_id(points, la.end);
                    let b1 = point_by_id(points, lb.start);
                    let b2 = point_by_id(points, lb.end);
                    a1.distance(a2) - b1.distance(b2)
                } else { 0.0 }
            }
            Constraint::Parallel { line_a, line_b } => {
                let la = line_by_id(lines, *line_a);
                let lb = line_by_id(lines, *line_b);
                if let (Some(la), Some(lb)) = (la, lb) {
                    let a1 = point_by_id(points, la.start);
                    let a2 = point_by_id(points, la.end);
                    let b1 = point_by_id(points, lb.start);
                    let b2 = point_by_id(points, lb.end);
                    let da = (a2 - a1).normalize();
                    let db = (b2 - b1).normalize();
                    da.perp_dot(db).abs()
                } else { 0.0 }
            }
            Constraint::Perpendicular { line_a, line_b } => {
                let la = line_by_id(lines, *line_a);
                let lb = line_by_id(lines, *line_b);
                if let (Some(la), Some(lb)) = (la, lb) {
                    let a1 = point_by_id(points, la.start);
                    let a2 = point_by_id(points, la.end);
                    let b1 = point_by_id(points, lb.start);
                    let b2 = point_by_id(points, lb.end);
                    let da = (a2 - a1).normalize();
                    let db = (b2 - b1).normalize();
                    da.dot(db).abs()
                } else { 0.0 }
            }
            Constraint::Midpoint { point, line } => {
                if let Some(l) = lines.iter().find(|l| l.id == *line) {
                    let a = point_by_id(points, l.start);
                    let b = point_by_id(points, l.end);
                    let mid = (a + b) * 0.5;
                    let p = point_by_id(points, *point);
                    p.distance(mid)
                } else { 0.0 }
            }
            Constraint::Tangent { .. } => 0.0,
            Constraint::Radius { .. } => 0.0,
            Constraint::Angle { line_a, line_b, angle } => {
                let la = line_by_id(lines, *line_a);
                let lb = line_by_id(lines, *line_b);
                if let (Some(la), Some(lb)) = (la, lb) {
                    let a1 = point_by_id(points, la.start);
                    let a2 = point_by_id(points, la.end);
                    let b1 = point_by_id(points, lb.start);
                    let b2 = point_by_id(points, lb.end);
                    let da = (a2 - a1).normalize();
                    let db = (b2 - b1).normalize();
                    let current = da.angle_to(db);
                    (current - *angle).abs()
                } else { 0.0 }
            }
            Constraint::Collinear { line_a, line_b } => {
                let la = line_by_id(lines, *line_a);
                let lb = line_by_id(lines, *line_b);
                if let (Some(la), Some(lb)) = (la, lb) {
                    let a1 = point_by_id(points, la.start);
                    let a2 = point_by_id(points, la.end);
                    let b1 = point_by_id(points, lb.start);
                    let b2 = point_by_id(points, lb.end);
                    let da = (a2 - a1).normalize();
                    let cross1 = da.perp_dot(b1 - a1).abs();
                    let cross2 = da.perp_dot(b2 - a1).abs();
                    cross1 + cross2
                } else { 0.0 }
            }
        }
    }
}

fn point_by_id(points: &[Point], id: EntityId) -> DVec2 {
    points.iter().find(|p| p.id == id).map(|p| p.pos()).unwrap_or(DVec2::ZERO)
}

fn line_by_id(lines: &[Line], id: EntityId) -> Option<&Line> {
    lines.iter().find(|l| l.id == id)
}
