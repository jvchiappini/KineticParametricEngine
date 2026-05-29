use glam::DVec2;
use crate::sketch::entities::*;
use crate::sketch::constraints::Constraint;

const MAX_ITER: usize = 2000;
const CONVERGENCE: f64 = 1e-8;
const INITIAL_RELAXATION: f64 = 0.3;
const MIN_RELAXATION: f64 = 0.05;
const ADAPTIVE_RATE: f64 = 0.95;
const MOMENTUM: f64 = 0.2;
const OSCILLATION_WINDOW: usize = 8;

pub struct Solver;

impl Solver {
    pub fn solve(
        points: &mut Vec<Point>,
        lines: &[Line],
        arcs: &[Arc],
        circles: &[Circle],
        constraints: &[Constraint],
    ) -> Result<(), String> {
        let vars = build_var_list(points, constraints);
        apply_fix(points, constraints);

        let mut relaxation = INITIAL_RELAXATION;
        let mut momentum_buf: Vec<DVec2> = vec![DVec2::ZERO; vars.len()];
        let mut osc_count = 0usize;
        let mut prev_error = 0.0;

        for _ in 0..MAX_ITER {
            let max_error = eval_all(points, lines, arcs, circles, constraints);
            if max_error < CONVERGENCE {
                return Ok(());
            }

            let mut updates: Vec<DVec2> = vec![DVec2::ZERO; vars.len()];

            for (vi, &(idx, fx, fy)) in vars.iter().enumerate() {
                if fx && fy { continue; }
                let orig = DVec2::new(points[idx].x, points[idx].y);
                let eps = 1e-6;

                for c in constraints {
                    if !affects_point(c, points[idx].id, lines) { continue; }

                    let err0 = Self::eval_one(points, lines, arcs, circles, c);
                    let mut grad = DVec2::ZERO;

                    if !fx {
                        points[idx].x = orig.x + eps;
                        let errx = Self::eval_one(points, lines, arcs, circles, c);
                        grad.x = (errx - err0) / eps;
                        points[idx].x = orig.x;
                    }
                    if !fy {
                        points[idx].y = orig.y + eps;
                        let erry = Self::eval_one(points, lines, arcs, circles, c);
                        grad.y = (erry - err0) / eps;
                        points[idx].y = orig.y;
                    }

                    let grad2 = grad.length_squared();
                    if grad2 > 1e-20 {
                        updates[vi] += grad * (-err0 / grad2);
                    }
                }
            }

            let mut total_step = 0.0;
            for (vi, &(idx, fx, fy)) in vars.iter().enumerate() {
                if fx && fy { continue; }
                let step = updates[vi];
                let step_len = step.length();
                if step_len < 1e-20 { continue; }
                total_step += step_len;

                let dot = step.dot(momentum_buf[vi]);
                if dot < 0.0 {
                    osc_count += 1;
                }

                let combined = step * relaxation + momentum_buf[vi] * MOMENTUM;
                points[idx].x += combined.x;
                points[idx].y += combined.y;
                momentum_buf[vi] = combined;
            }

            if osc_count > OSCILLATION_WINDOW {
                relaxation = (relaxation * ADAPTIVE_RATE).max(MIN_RELAXATION);
                osc_count = 0;
            }

            if total_step < 1e-14 && max_error > CONVERGENCE {
                if max_error > prev_error && relaxation > MIN_RELAXATION {
                    relaxation *= 0.5;
                }
                break;
            }
            prev_error = max_error;
        }

        let final_err = eval_all(points, lines, arcs, circles, constraints);
        if final_err > 1.0 {
            Err(format!("Solver did not converge (error: {:.6})", final_err))
        } else {
            Ok(())
        }
    }

    fn eval_one(points: &[Point], lines: &[Line], arcs: &[Arc], circles: &[Circle], c: &Constraint) -> f64 {
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
            Constraint::Tangent { line, arc } => {
                if let Some(l) = lines.iter().find(|l| l.id == *line) {
                    let a = point_by_id(points, l.start);
                    let b = point_by_id(points, l.end);
                    let dir = (b - a).normalize();
                    let line_len = a.distance(b);
                    if line_len < 1e-12 { return 0.0; }
                    let arc_data = arcs.iter().find(|a| a.id == *arc);
                    if let Some(arc_data) = arc_data {
                        let center = point_by_id(points, arc_data.center);
                        if let Some(circle) = circles.iter().find(|c| c.id == *arc) {
                            let to_center = center - a;
                            let dist = to_center.perp_dot(dir).abs();
                            (dist - circle.radius).abs()
                        } else {
                            let to_center = center - a;
                            let dist = to_center.perp_dot(dir).abs();
                            (dist - arc_data.radius).abs()
                        }
                    } else if let Some(circle) = circles.iter().find(|c| c.id == *arc) {
                        if let Some(center) = points.iter().find(|p| p.id == circle.center) {
                            let to_center = center.pos() - a;
                            let dist = to_center.perp_dot(dir).abs();
                            (dist - circle.radius).abs()
                        } else { 0.0 }
                    } else { 0.0 }
                } else { 0.0 }
            }
            Constraint::Radius { arc_or_circle, radius } => {
                if let Some(arc_data) = arcs.iter().find(|a| a.id == *arc_or_circle) {
                    (arc_data.radius - radius).abs()
                } else if let Some(circle) = circles.iter().find(|c| c.id == *arc_or_circle) {
                    (circle.radius - radius).abs()
                } else { 0.0 }
            }
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

fn eval_all(points: &[Point], lines: &[Line], arcs: &[Arc], circles: &[Circle], constraints: &[Constraint]) -> f64 {
    let mut max_err: f64 = 0.0;
    for c in constraints {
        let err = Solver::eval_one(points, lines, arcs, circles, c);
        max_err = f64::max(max_err, err.abs());
    }
    max_err
}

fn build_var_list(points: &[Point], constraints: &[Constraint]) -> Vec<(usize, bool, bool)> {
    points.iter().enumerate().map(|(i, pt)| {
        let mut fixed_x = false;
        let mut fixed_y = false;
        for c in constraints {
            if let Constraint::Fix { point, .. } = c {
                if *point == pt.id {
                    fixed_x = true;
                    fixed_y = true;
                }
            }
        }
        (i, fixed_x, fixed_y)
    }).collect()
}

fn apply_fix(points: &mut [Point], constraints: &[Constraint]) {
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
}

fn affects_point(c: &Constraint, point_id: u64, lines: &[Line]) -> bool {
    match c {
        Constraint::Horizontal { line }
        | Constraint::Vertical { line } => {
            lines.iter().any(|l| l.id == *line && (l.start == point_id || l.end == point_id))
        }
        Constraint::Coincident { point_a, point_b } => *point_a == point_id || *point_b == point_id,
        Constraint::Fix { point, .. } => *point == point_id,
        Constraint::Distance { point_a, point_b, .. } => *point_a == point_id || *point_b == point_id,
        Constraint::EqualLength { line_a, line_b } => {
            lines.iter().any(|l| (l.id == *line_a || l.id == *line_b) && (l.start == point_id || l.end == point_id))
        }
        Constraint::Parallel { line_a, line_b }
        | Constraint::Perpendicular { line_a, line_b }
        | Constraint::Collinear { line_a, line_b } => {
            lines.iter().any(|l| (l.id == *line_a || l.id == *line_b) && (l.start == point_id || l.end == point_id))
        }
        Constraint::Midpoint { point, line: _ } => *point == point_id,
        Constraint::Tangent { line, arc: _ } => {
            lines.iter().any(|l| l.id == *line && (l.start == point_id || l.end == point_id))
        }
        Constraint::Radius { .. } => false,
        Constraint::Angle { line_a, line_b, .. } => {
            lines.iter().any(|l| (l.id == *line_a || l.id == *line_b) && (l.start == point_id || l.end == point_id))
        }
    }
}

fn point_by_id(points: &[Point], id: u64) -> DVec2 {
    points.iter().find(|p| p.id == id).map(|p| p.pos()).unwrap_or(DVec2::ZERO)
}

fn line_by_id(lines: &[Line], id: u64) -> Option<&Line> {
    lines.iter().find(|l| l.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pt(id: u64, x: f64, y: f64) -> Point {
        Point { id, x, y }
    }

    fn make_line(id: u64, start: u64, end: u64) -> Line {
        Line { id, start, end }
    }

    fn error_leq(points: &[Point], lines: &[Line], arcs: &[Arc], circles: &[Circle], constraints: &[Constraint], tol: f64) -> bool {
        for c in constraints {
            let err = Solver::eval_one(points, lines, arcs, circles, c);
            if err.abs() > tol { return false; }
        }
        true
    }

    #[test]
    fn test_horizontal_constraint() {
        let mut points = vec![make_pt(1, 0.0, 0.0), make_pt(2, 3.0, 2.0)];
        let lines = vec![make_line(10, 1, 2)];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![Constraint::Horizontal { line: 10 }];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!((points[1].y - points[0].y).abs() < 1e-6);
    }

    #[test]
    fn test_vertical_constraint() {
        let mut points = vec![make_pt(1, 0.0, 0.0), make_pt(2, 2.0, 3.0)];
        let lines = vec![make_line(10, 1, 2)];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![Constraint::Vertical { line: 10 }];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!((points[1].x - points[0].x).abs() < 1e-6);
    }

    #[test]
    fn test_fix_and_distance() {
        let mut points = vec![make_pt(1, 0.0, 0.0), make_pt(2, 5.0, 5.0)];
        let lines = vec![make_line(10, 1, 2)];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![
            Constraint::Fix { point: 1, x: 0.0, y: 0.0 },
            Constraint::Distance { point_a: 1, point_b: 2, distance: 3.0 },
        ];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        let d = points[1].pos().distance(points[0].pos());
        assert!((d - 3.0).abs() < 1e-6);
        assert!((points[0].x - 0.0).abs() < 1e-6);
        assert!((points[0].y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_rectangle_cyclic() {
        let mut points = vec![
            make_pt(1, 0.0, 0.0),
            make_pt(2, 4.0, 0.5),
            make_pt(3, 4.5, 3.0),
            make_pt(4, -0.5, 2.5),
        ];
        let lines = vec![
            make_line(10, 1, 2),
            make_line(11, 2, 3),
            make_line(12, 3, 4),
            make_line(13, 4, 1),
        ];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![
            Constraint::Fix { point: 1, x: 0.0, y: 0.0 },
            Constraint::Horizontal { line: 10 },
            Constraint::Horizontal { line: 12 },
            Constraint::Vertical { line: 11 },
            Constraint::Vertical { line: 13 },
            Constraint::Distance { point_a: 1, point_b: 2, distance: 5.0 },
            Constraint::Distance { point_a: 2, point_b: 3, distance: 3.0 },
        ];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!(error_leq(&points, &lines, &arcs, &circles, &constraints, 1e-6));
        assert!((points[1].y - points[0].y).abs() < 1e-6);
        assert!((points[2].x - points[1].x).abs() < 1e-6);
        assert!((points[3].x - points[0].x).abs() < 1e-6);
        let d1 = points[1].x - points[0].x;
        assert!((d1 - 5.0).abs() < 1e-6);
        let d2 = points[2].y - points[1].y;
        assert!((d2 - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_triangle_cyclic() {
        let mut points = vec![
            make_pt(1, 0.0, 0.0),
            make_pt(2, 4.0, 1.0),
            make_pt(3, 2.0, 3.0),
        ];
        let lines = vec![
            make_line(10, 1, 2),
            make_line(11, 2, 3),
            make_line(12, 3, 1),
        ];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![
            Constraint::Fix { point: 1, x: 0.0, y: 0.0 },
            Constraint::Fix { point: 2, x: 5.0, y: 0.0 },
            Constraint::Distance { point_a: 2, point_b: 3, distance: 4.0 },
            Constraint::Distance { point_a: 3, point_b: 1, distance: 3.0 },
        ];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!(error_leq(&points, &lines, &arcs, &circles, &constraints, 1e-6));
        assert!((points[0].x - 0.0).abs() < 1e-6);
        assert!((points[0].y - 0.0).abs() < 1e-6);
        assert!((points[1].x - 5.0).abs() < 1e-6);
        assert!((points[1].y - 0.0).abs() < 1e-6);
        let d23 = points[2].pos().distance(points[1].pos());
        assert!((d23 - 4.0).abs() < 1e-6);
        let d31 = points[2].pos().distance(points[0].pos());
        assert!((d31 - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_coincident() {
        let mut points = vec![make_pt(1, 1.0, 2.0), make_pt(2, 5.0, 6.0)];
        let lines = vec![];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![
            Constraint::Coincident { point_a: 1, point_b: 2 },
        ];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        let d = points[0].pos().distance(points[1].pos());
        assert!(d < 1e-6);
    }

    #[test]
    fn test_parallel_constraint() {
        let mut points = vec![
            make_pt(1, 0.0, 0.0), make_pt(2, 2.0, 2.0),
            make_pt(3, 1.0, 0.0), make_pt(4, 4.0, 1.0),
        ];
        let lines = vec![make_line(10, 1, 2), make_line(11, 3, 4)];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![Constraint::Parallel { line_a: 10, line_b: 11 }];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!(error_leq(&points, &lines, &arcs, &circles, &constraints, 1e-6));
    }

    #[test]
    fn test_cyclic_stress() {
        let mut points = vec![
            make_pt(1, 0.0, 0.0),
            make_pt(2, 3.0, 1.0),
            make_pt(3, 4.0, 4.0),
            make_pt(4, -1.0, 3.0),
            make_pt(5, 2.0, -1.0),
        ];
        let lines = vec![
            make_line(10, 1, 2),
            make_line(11, 2, 3),
            make_line(12, 3, 4),
            make_line(13, 4, 5),
            make_line(14, 5, 1),
        ];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![
            Constraint::Fix { point: 1, x: 0.0, y: 0.0 },
            Constraint::Distance { point_a: 1, point_b: 2, distance: 3.0 },
            Constraint::Distance { point_a: 2, point_b: 3, distance: 4.0 },
            Constraint::Distance { point_a: 3, point_b: 4, distance: 5.0 },
            Constraint::Distance { point_a: 4, point_b: 5, distance: 2.0 },
            Constraint::Distance { point_a: 5, point_b: 1, distance: 3.5 },
            Constraint::Parallel { line_a: 10, line_b: 12 },
            Constraint::Perpendicular { line_a: 11, line_b: 13 },
        ];
        let result = Solver::solve(&mut points, &lines, &arcs, &circles, &constraints);
        assert!(result.is_ok(), "Solver failed: {:?}", result);
        assert!(error_leq(&points, &lines, &arcs, &circles, &constraints, 1e-4));
    }
}
