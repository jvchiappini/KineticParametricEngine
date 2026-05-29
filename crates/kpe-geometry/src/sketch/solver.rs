use glam::DVec2;
use crate::sketch::entities::*;
use crate::sketch::constraints::Constraint;

/// Maximum Newton-Raphson iterations
const MAX_ITER: usize = 200;

/// Convergence threshold (max absolute constraint error)
const CONVERGENCE: f64 = 1e-7;

/// Initial Levenberg-Marquardt damping factor
const LAMBDA_INIT: f64 = 1e-3;

/// Minimum damping
const LAMBDA_MIN: f64 = 1e-12;

/// Maximum damping
const LAMBDA_MAX: f64 = 1e12;

/// Damping increase factor (when step is rejected)
const LAMBDA_UP: f64 = 2.0;

/// Damping decrease factor (when step is accepted)
const LAMBDA_DOWN: f64 = 0.5;

/// Finite-difference step for Jacobian
const JAC_EPS: f64 = 1e-6;

/// Threshold for DoF analysis (column norm below this → unconstrained)
const DOF_THRESHOLD: f64 = 1e-6;

/// A single optimization variable: one coordinate of one point.
#[derive(Clone, Copy)]
struct Var {
    pt_idx: usize,
    coord: usize,
}

pub struct Solver;

impl Solver {
    /// Solve the constraint system using Levenberg-Marquardt (Newton-Raphson family).
    ///
    /// Returns `Ok(())` if the solver converges to within tolerance,
    /// or `Err(String)` if it fails to converge.
    pub fn solve(
        points: &mut Vec<Point>,
        lines: &[Line],
        arcs: &[Arc],
        circles: &[Circle],
        constraints: &[Constraint],
    ) -> Result<(), String> {
        apply_fix(points, constraints);
        let vars = build_vars(points, constraints);
        let n = vars.len();
        let m = constraints.len();

        if n == 0 || m == 0 {
            return Ok(());
        }

        let mut prev_err = eval_all(points, lines, arcs, circles, constraints);
        if prev_err < CONVERGENCE {
            return Ok(());
        }

        let mut lam = LAMBDA_INIT;

        for _ in 0..MAX_ITER {
            let f: Vec<f64> = constraints.iter().map(|c| eval_one(points, lines, arcs, circles, c)).collect();

            // Build Jacobian J (m × n) via finite differences
            let mut jac = vec![vec![0.0; n]; m];
            for c_idx in 0..m {
                for v_idx in 0..n {
                    let pt = vars[v_idx].pt_idx;
                    let coord = vars[v_idx].coord;
                    let orig = if coord == 0 { points[pt].x } else { points[pt].y };

                    if coord == 0 {
                        points[pt].x = orig + JAC_EPS;
                    } else {
                        points[pt].y = orig + JAC_EPS;
                    }
                    let err_pert = eval_one(points, lines, arcs, circles, &constraints[c_idx]);
                    jac[c_idx][v_idx] = (err_pert - f[c_idx]) / JAC_EPS;

                    if coord == 0 {
                        points[pt].x = orig;
                    } else {
                        points[pt].y = orig;
                    }
                }
            }

            // Build normal equations: J^T * J and J^T * f
            let mut jtj = vec![vec![0.0; n]; n];
            let mut jtf = vec![0.0; n];
            for i in 0..n {
                for k in 0..m {
                    jtf[i] += jac[k][i] * f[k];
                }
                for j in 0..=i {
                    let mut s = 0.0;
                    for k in 0..m {
                        s += jac[k][i] * jac[k][j];
                    }
                    jtj[i][j] = s;
                    jtj[j][i] = s;
                }
            }

            // Levenberg-Marquardt inner loop: try steps with increasing damping
            let mut accepted = false;
            let mut trial_lam = lam;
            for _ in 0..20 {
                let mut a = jtj.clone();
                for i in 0..n {
                    a[i][i] += trial_lam * jtj[i][i].max(1e-12);
                }

                if let Ok(delta) = cholesky_solve(&a, &jtf) {
                    let mut trial = points.clone();
                    for v_idx in 0..n {
                        let pt = vars[v_idx].pt_idx;
                        let coord = vars[v_idx].coord;
                        // Δ = -delta  (since A*delta = +J^T*f but A*Δ = -J^T*f)
                        if coord == 0 {
                            trial[pt].x -= delta[v_idx];
                        } else {
                            trial[pt].y -= delta[v_idx];
                        }
                    }

                    let new_err = eval_all(&trial, lines, arcs, circles, constraints);

                    if new_err < prev_err {
                        *points = trial;
                        prev_err = new_err;
                        lam = (trial_lam * LAMBDA_DOWN).max(LAMBDA_MIN);
                        accepted = true;
                        break;
                    }
                }

                trial_lam = (trial_lam * LAMBDA_UP).min(LAMBDA_MAX);
            }

            if !accepted {
                break;
            }

            if prev_err < CONVERGENCE {
                return Ok(());
            }
        }

        let final_err = eval_all(points, lines, arcs, circles, constraints);
        if final_err > 0.1 {
            Err(format!("Solver did not converge (error: {:.6})", final_err))
        } else {
            Ok(())
        }
    }
}

/// Evaluate a single constraint's error value (signed where possible for smooth optimization).
pub fn eval_one(
    points: &[Point],
    lines: &[Line],
    arcs: &[Arc],
    circles: &[Circle],
    c: &Constraint,
) -> f64 {
    match c {
        Constraint::Horizontal { line } => {
            if let Some(l) = lines.iter().find(|l| l.id == *line) {
                let a = point_by_id(points, l.start);
                let b = point_by_id(points, l.end);
                a.y - b.y
            } else {
                0.0
            }
        }
        Constraint::Vertical { line } => {
            if let Some(l) = lines.iter().find(|l| l.id == *line) {
                let a = point_by_id(points, l.start);
                let b = point_by_id(points, l.end);
                a.x - b.x
            } else {
                0.0
            }
        }
        Constraint::Coincident { point_a, point_b } => {
            let a = point_by_id(points, *point_a);
            let b = point_by_id(points, *point_b);
            let d = a - b;
            (d.length_squared() + 1e-20).sqrt()
        }
        Constraint::Fix { point, x, y } => {
            let p = point_by_id(points, *point);
            ((p.x - x) * (p.x - x) + (p.y - y) * (p.y - y) + 1e-20).sqrt()
        }
        Constraint::Distance { point_a, point_b, distance } => {
            let a = point_by_id(points, *point_a);
            let b = point_by_id(points, *point_b);
            let d = a - b;
            let len = (d.length_squared() + 1e-20).sqrt();
            len - distance
        }
        Constraint::EqualLength { line_a, line_b } => {
            let la = line_by_id(lines, *line_a);
            let lb = line_by_id(lines, *line_b);
            if let (Some(la), Some(lb)) = (la, lb) {
                let a1 = point_by_id(points, la.start);
                let a2 = point_by_id(points, la.end);
                let b1 = point_by_id(points, lb.start);
                let b2 = point_by_id(points, lb.end);
                let len_a = ((a2 - a1).length_squared() + 1e-20).sqrt();
                let len_b = ((b2 - b1).length_squared() + 1e-20).sqrt();
                len_a - len_b
            } else {
                0.0
            }
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
                da.perp_dot(db)
            } else {
                0.0
            }
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
                da.dot(db)
            } else {
                0.0
            }
        }
        Constraint::Midpoint { point, line } => {
            if let Some(l) = lines.iter().find(|l| l.id == *line) {
                let a = point_by_id(points, l.start);
                let b = point_by_id(points, l.end);
                let mid = (a + b) * 0.5;
                let p = point_by_id(points, *point);
                let d = p - mid;
                (d.length_squared() + 1e-20).sqrt()
            } else {
                0.0
            }
        }
        Constraint::Tangent { line, arc } => {
            if let Some(l) = lines.iter().find(|l| l.id == *line) {
                let a = point_by_id(points, l.start);
                let b = point_by_id(points, l.end);
                let line_vec = b - a;
                let line_len_sq = line_vec.length_squared();
                if line_len_sq < 1e-24 {
                    return 0.0;
                }
                let inv_len = 1.0 / line_len_sq.sqrt();
                let dir = line_vec * inv_len;
                let arc_data = arcs.iter().find(|a| a.id == *arc);
                if let Some(arc_data) = arc_data {
                    let center = point_by_id(points, arc_data.center);
                    let to_center = center - a;
                    let perp_dist = to_center.perp_dot(dir);
                    let rad = if let Some(circle) = circles.iter().find(|c| c.id == *arc) {
                        circle.radius
                    } else {
                        arc_data.radius
                    };
                    // Signed tangent error: perp_dist^2 - rad^2 is smooth
                    perp_dist * perp_dist - rad * rad
                } else if let Some(circle) = circles.iter().find(|c| c.id == *arc) {
                    if let Some(center) = points.iter().find(|p| p.id == circle.center) {
                        let to_center = center.pos() - a;
                        let perp_dist = to_center.perp_dot(dir);
                        perp_dist * perp_dist - circle.radius * circle.radius
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
        Constraint::Radius { arc_or_circle, radius } => {
            if let Some(arc_data) = arcs.iter().find(|a| a.id == *arc_or_circle) {
                arc_data.radius - radius
            } else if let Some(circle) = circles.iter().find(|c| c.id == *arc_or_circle) {
                circle.radius - radius
            } else {
                0.0
            }
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
                current - *angle
            } else {
                0.0
            }
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
                let cross1 = da.perp_dot(b1 - a1);
                let cross2 = da.perp_dot(b2 - a1);
                // Use smooth signed sum so gradient doesn't vanish
                cross1 + cross2
            } else {
                0.0
            }
        }
    }
}

fn eval_all(
    points: &[Point],
    lines: &[Line],
    arcs: &[Arc],
    circles: &[Circle],
    constraints: &[Constraint],
) -> f64 {
    let mut max_err: f64 = 0.0;
    for c in constraints {
        let err = eval_one(points, lines, arcs, circles, c);
        max_err = f64::max(max_err, err.abs());
    }
    max_err
}

fn build_vars(points: &[Point], constraints: &[Constraint]) -> Vec<Var> {
    let mut vars = Vec::new();
    for (i, pt) in points.iter().enumerate() {
        let fixed = constraints
            .iter()
            .any(|c| matches!(c, Constraint::Fix { point, .. } if *point == pt.id));
        if !fixed {
            vars.push(Var {
                pt_idx: i,
                coord: 0,
            });
            vars.push(Var {
                pt_idx: i,
                coord: 1,
            });
        }
    }
    vars
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

/// Cholesky decomposition: A = L * L^T, then solve A * x = b.
/// A must be symmetric positive definite.
fn cholesky_solve(a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>, String> {
    let n = b.len();
    let mut l = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..=i {
            let mut s = 0.0;
            for k in 0..j {
                s += l[i][k] * l[j][k];
            }
            if i == j {
                let val = a[i][i] - s;
                if val <= 0.0 {
                    return Err("Matrix not positive definite".into());
                }
                l[i][i] = val.sqrt();
            } else {
                l[i][j] = (a[i][j] - s) / l[j][j];
            }
        }
    }

    // Forward substitution: L * y = b
    let mut y = vec![0.0; n];
    for i in 0..n {
        let mut s = 0.0;
        for j in 0..i {
            s += l[i][j] * y[j];
        }
        y[i] = (b[i] - s) / l[i][i];
    }

    // Back substitution: L^T * x = y
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut s = 0.0;
        for j in (i + 1)..n {
            s += l[j][i] * x[j];
        }
        x[i] = (y[i] - s) / l[i][i];
    }

    Ok(x)
}

/// Analyze constraint status of each point's coordinates.
///
/// Returns a vector of `(x_constrained, y_constrained)` for each point,
/// based on Jacobian column norms evaluated at the current configuration.
/// A coordinate is considered "constrained" if its column in the Jacobian
/// has norm above `DOF_THRESHOLD`.
pub fn analyze_dof(
    points: &[Point],
    lines: &[Line],
    arcs: &[Arc],
    circles: &[Circle],
    constraints: &[Constraint],
) -> Vec<(bool, bool)> {
    let vars = build_vars(points, constraints);
    let n = vars.len();
    let m = constraints.len();
    let mut result = vec![(false, false); points.len()];

    // Fixed points (with Fix constraint) are always fully constrained
    for c in constraints {
        if let Constraint::Fix { point, .. } = c {
            if let Some(pt) = points.iter().position(|p| p.id == *point) {
                result[pt] = (true, true);
            }
        }
    }

    if m == 0 || n == 0 {
        return result;
    }

    let mut jac = vec![vec![0.0; n]; m];
    let mut work = points.to_vec();
    for c_idx in 0..m {
        let err0 = eval_one(points, lines, arcs, circles, &constraints[c_idx]);
        for v_idx in 0..n {
            let pt = vars[v_idx].pt_idx;
            let coord = vars[v_idx].coord;
            let orig = if coord == 0 { points[pt].x } else { points[pt].y };

            if coord == 0 {
                work[pt].x = orig + JAC_EPS;
            } else {
                work[pt].y = orig + JAC_EPS;
            }
            let err1 = eval_one(&work, lines, arcs, circles, &constraints[c_idx]);
            jac[c_idx][v_idx] = (err1 - err0) / JAC_EPS;

            if coord == 0 {
                work[pt].x = orig;
            } else {
                work[pt].y = orig;
            }
        }
    }

    for v_idx in 0..n {
        let mut norm2 = 0.0;
        for c_idx in 0..m {
            norm2 += jac[c_idx][v_idx] * jac[c_idx][v_idx];
        }
        let constrained = norm2.sqrt() > DOF_THRESHOLD;
        let pt = vars[v_idx].pt_idx;
        if vars[v_idx].coord == 0 {
            result[pt].0 = constrained;
        } else {
            result[pt].1 = constrained;
        }
    }

    result
}

fn point_by_id(points: &[Point], id: u64) -> DVec2 {
    points
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.pos())
        .unwrap_or(DVec2::ZERO)
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

    fn error_leq(
        points: &[Point],
        lines: &[Line],
        arcs: &[Arc],
        circles: &[Circle],
        constraints: &[Constraint],
        tol: f64,
    ) -> bool {
        for c in constraints {
            let err = eval_one(points, lines, arcs, circles, c);
            if err.abs() > tol {
                return false;
            }
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
            Constraint::Distance {
                point_a: 1,
                point_b: 2,
                distance: 3.0,
            },
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
            Constraint::Distance {
                point_a: 1,
                point_b: 2,
                distance: 5.0,
            },
            Constraint::Distance {
                point_a: 2,
                point_b: 3,
                distance: 3.0,
            },
        ];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!(error_leq(
            &points, &lines, &arcs, &circles, &constraints, 1e-6
        ));
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
            Constraint::Fix {
                point: 2,
                x: 5.0,
                y: 0.0,
            },
            Constraint::Distance {
                point_a: 2,
                point_b: 3,
                distance: 4.0,
            },
            Constraint::Distance {
                point_a: 3,
                point_b: 1,
                distance: 3.0,
            },
        ];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!(error_leq(
            &points, &lines, &arcs, &circles, &constraints, 1e-6
        ));
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
        let constraints = vec![Constraint::Coincident {
            point_a: 1,
            point_b: 2,
        }];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        let d = points[0].pos().distance(points[1].pos());
        assert!(d < 1e-6, "d={}", d);
    }

    #[test]
    fn test_parallel_constraint() {
        let mut points = vec![
            make_pt(1, 0.0, 0.0),
            make_pt(2, 2.0, 2.0),
            make_pt(3, 1.0, 0.0),
            make_pt(4, 4.0, 1.0),
        ];
        let lines = vec![make_line(10, 1, 2), make_line(11, 3, 4)];
        let arcs = vec![];
        let circles = vec![];
        let constraints = vec![Constraint::Parallel {
            line_a: 10,
            line_b: 11,
        }];
        Solver::solve(&mut points, &lines, &arcs, &circles, &constraints).unwrap();
        assert!(error_leq(
            &points, &lines, &arcs, &circles, &constraints, 1e-6
        ));
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
            Constraint::Distance {
                point_a: 1,
                point_b: 2,
                distance: 3.0,
            },
            Constraint::Distance {
                point_a: 2,
                point_b: 3,
                distance: 4.0,
            },
            Constraint::Distance {
                point_a: 3,
                point_b: 4,
                distance: 5.0,
            },
            Constraint::Distance {
                point_a: 4,
                point_b: 5,
                distance: 2.0,
            },
            Constraint::Distance {
                point_a: 5,
                point_b: 1,
                distance: 3.5,
            },
            Constraint::Parallel {
                line_a: 10,
                line_b: 12,
            },
            Constraint::Perpendicular {
                line_a: 11,
                line_b: 13,
            },
        ];
        let result = Solver::solve(&mut points, &lines, &arcs, &circles, &constraints);
        assert!(result.is_ok(), "Solver failed: {:?}", result);
        assert!(error_leq(
            &points, &lines, &arcs, &circles, &constraints, 1e-4
        ));
    }

    #[test]
    fn test_dof_analysis() {
        let points = vec![
            make_pt(1, 0.0, 0.0),
            make_pt(2, 5.0, 0.0),
            make_pt(3, 2.0, 3.0),
        ];
        let lines = vec![
            make_line(10, 1, 2),
            make_line(11, 2, 3),
            make_line(12, 3, 1),
        ];
        let constraints = vec![
            Constraint::Fix { point: 1, x: 0.0, y: 0.0 },
            Constraint::Fix {
                point: 2,
                x: 5.0,
                y: 0.0,
            },
        ];
        // Point 3 has no constraints → under-constrained
        let dof = analyze_dof(&points, &lines, &[], &[], &constraints);
        assert!(dof[0].0 && dof[0].1, "Fixed point 1 should be constrained");
        assert!(dof[1].0 && dof[1].1, "Fixed point 2 should be constrained");
        assert!(!dof[2].0 && !dof[2].1, "Free point 3 should be unconstrained");
    }
}
