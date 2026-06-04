use glam::DVec2;

/// Epsilon for floating-point comparisons.
const EPS: f64 = 1e-9;

/// Solve quadratic `a·t² + b·t + c = 0` and return real roots in [0,1].
fn solve_quadratic(a: f64, b: f64, c: f64) -> Vec<f64> {
    if a.abs() < EPS {
        if b.abs() < EPS {
            return vec![];
        }
        let t = -c / b;
        return if (0.0..=1.0).contains(&t) { vec![t] } else { vec![] };
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return vec![];
    }
    let sqrt_disc = disc.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);
    let mut roots = Vec::with_capacity(2);
    if (0.0..=1.0).contains(&t1) { roots.push(t1); }
    if (0.0..=1.0).contains(&t2) { roots.push(t2); }
    roots
}

/// Return the smallest positive angle (0..TAU) for a given point relative to
/// a center.  Used to check arc angular ranges.
fn angle_of(center: DVec2, p: DVec2) -> f64 {
    let d = p - center;
    let mut a = d.y.atan2(d.x);
    if a < 0.0 { a += std::f64::consts::TAU; }
    a
}

/// Check whether a point's angle (relative to `center`) falls within the
/// arc's sweep range [start_angle, start_angle + sweep_angle].
fn angle_in_range(center: DVec2, p: DVec2, start_angle: f64, sweep_angle: f64) -> bool {
    let a = angle_of(center, p);
    let start = start_angle;
    let end = start + sweep_angle;
    if sweep_angle >= 0.0 {
        a >= start - EPS && a <= end + EPS
    } else {
        a >= end - EPS && a <= start + EPS
    }
}

/// Intersection of two line segments (a1–a2 and b1–b2).
/// Returns the intersection point, or `None` if they are parallel or do not
/// intersect within the segment bounds.
pub fn line_line(a1: DVec2, a2: DVec2, b1: DVec2, b2: DVec2) -> Option<DVec2> {
    let r = a2 - a1;
    let s = b2 - b1;
    let rxs = r.perp_dot(s);
    if rxs.abs() < EPS {
        return None;
    }
    let t = (b1 - a1).perp_dot(s) / rxs;
    let u = (b1 - a1).perp_dot(r) / rxs;
    if t >= -EPS && t <= 1.0 + EPS && u >= -EPS && u <= 1.0 + EPS {
        Some(a1 + r * t)
    } else {
        None
    }
}

/// Intersection of a line segment (p1–p2) with a circle (center, radius).
/// Returns 0, 1, or 2 intersection points that lie on the segment.
pub fn line_circle(p1: DVec2, p2: DVec2, center: DVec2, radius: f64) -> Vec<DVec2> {
    let d = p2 - p1;
    let f = p1 - center;
    let a = d.dot(d);
    let b = 2.0 * f.dot(d);
    let c = f.dot(f) - radius * radius;
    let ts = solve_quadratic(a, b, c);
    ts.into_iter().map(|t| p1 + d * t).collect()
}

/// Intersection of a line segment with an arc.
/// Arc defined by (center, radius, start_angle, sweep_angle).
pub fn line_arc(
    p1: DVec2, p2: DVec2,
    center: DVec2, radius: f64,
    start_angle: f64, sweep_angle: f64,
) -> Vec<DVec2> {
    let pts = line_circle(p1, p2, center, radius);
    pts.into_iter()
        .filter(|p| angle_in_range(center, *p, start_angle, sweep_angle))
        .collect()
}

/// Intersection of two circles.
pub fn circle_circle(c1: DVec2, r1: f64, c2: DVec2, r2: f64) -> Vec<DVec2> {
    let d_vec = c2 - c1;
    let d = d_vec.length();
    if d < EPS || d > r1 + r2 + EPS || d < (r1 - r2).abs() - EPS {
        return vec![];
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h_sq = r1 * r1 - a * a;
    if h_sq < 0.0 {
        return vec![DVec2::new(c1.x + d_vec.x * a / d, c1.y + d_vec.y * a / d)];
    }
    let h = h_sq.sqrt();
    let perp = DVec2::new(-d_vec.y, d_vec.x) * (h / d);
    let along = d_vec * (a / d);
    let mid = c1 + along;
    vec![mid + perp, mid - perp]
}

/// Intersection of a circle and an arc.
pub fn circle_arc(
    c_center: DVec2, c_radius: f64,
    a_center: DVec2, a_radius: f64,
    a_start: f64, a_sweep: f64,
) -> Vec<DVec2> {
    let pts = circle_circle(c_center, c_radius, a_center, a_radius);
    pts.into_iter()
        .filter(|p| angle_in_range(a_center, *p, a_start, a_sweep))
        .collect()
}

/// Intersection of two arcs.
pub fn arc_arc(
    c1: DVec2, r1: f64, s1: f64, sw1: f64,
    c2: DVec2, r2: f64, s2: f64, sw2: f64,
) -> Vec<DVec2> {
    let pts = circle_circle(c1, r1, c2, r2);
    pts.into_iter()
        .filter(|p| angle_in_range(c1, *p, s1, sw1) && angle_in_range(c2, *p, s2, sw2))
        .collect()
}

/// Tangent points from an external point `p` to a circle `(center, radius)`.
///
/// Returns 0 or 2 tangent points (points of tangency on the circle).
/// Returns empty if `p` is inside or on the circle.
pub fn tangent_point_to_circle(p: DVec2, center: DVec2, radius: f64) -> Vec<DVec2> {
    let v = p - center;
    let d = v.length();
    if d <= radius + EPS {
        return vec![];
    }

    let ratio = radius / d;
    let ratio_sq = ratio * ratio;
    let along = v * ratio_sq;                    // v * R²/d²
    let perp_mag = ratio * (1.0 - ratio_sq).sqrt(); // R/d * √(1 - R²/d²)
    let perp = DVec2::new(-v.y, v.x) * perp_mag;

    vec![center + along + perp, center + along - perp]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_line_intersecting() {
        let p = line_line(
            DVec2::new(0.0, 0.0), DVec2::new(10.0, 0.0),
            DVec2::new(5.0, -5.0), DVec2::new(5.0, 5.0),
        );
        assert!(p.is_some());
        assert!((p.unwrap() - DVec2::new(5.0, 0.0)).length() < EPS);
    }

    #[test]
    fn test_line_line_parallel() {
        let p = line_line(
            DVec2::new(0.0, 0.0), DVec2::new(10.0, 0.0),
            DVec2::new(0.0, 5.0), DVec2::new(10.0, 5.0),
        );
        assert!(p.is_none());
    }

    #[test]
    fn test_line_circle_two_intersections() {
        let pts = line_circle(
            DVec2::new(-10.0, 0.0), DVec2::new(10.0, 0.0),
            DVec2::new(0.0, 0.0), 5.0,
        );
        assert_eq!(pts.len(), 2);
        assert!((pts[0].x + 5.0).abs() < EPS || (pts[0].x - 5.0).abs() < EPS);
        assert!((pts[1].x + 5.0).abs() < EPS || (pts[1].x - 5.0).abs() < EPS);
    }

    #[test]
    fn test_line_circle_tangent() {
        let pts = line_circle(
            DVec2::new(0.0, -10.0), DVec2::new(0.0, 10.0),
            DVec2::new(0.0, 0.0), 5.0,
        );
        assert_eq!(pts.len(), 2);
        for p in &pts {
            assert!((p.x - 0.0).abs() < EPS);
        }
    }

    #[test]
    fn test_circle_circle_two_intersections() {
        let pts = circle_circle(
            DVec2::new(0.0, 0.0), 5.0,
            DVec2::new(5.0, 0.0), 5.0,
        );
        assert_eq!(pts.len(), 2);
    }

    #[test]
    fn test_circle_circle_separate() {
        let pts = circle_circle(
            DVec2::new(0.0, 0.0), 1.0,
            DVec2::new(10.0, 0.0), 1.0,
        );
        assert!(pts.is_empty());
    }

    #[test]
    fn test_tangent_point_outside() {
        let pts = tangent_point_to_circle(
            DVec2::new(10.0, 0.0),
            DVec2::new(0.0, 0.0), 5.0,
        );
        // P outside circle (10,0), center (0,0) radius 5
        // Should have 2 tangent points
        assert_eq!(pts.len(), 2, "Expected 2 tangents, got {}", pts.len());
        for t in &pts {
            assert!((t.length() - 5.0).abs() < 1e-6, "Tangent point not on circle");
        }
    }

    #[test]
    fn test_tangent_point_inside() {
        let pts = tangent_point_to_circle(
            DVec2::new(0.0, 0.0),
            DVec2::new(0.0, 0.0), 5.0,
        );
        assert!(pts.is_empty());
    }

    #[test]
    fn test_line_arc_filters_by_angle() {
        // Arc: center (0,0), r=5, from 0° to 180° (upper half)
        // Line through from (-10, 0) to (10, 0) should intersect at two circle
        // points but one is filtered out (at 180° is on boundary).
        let pts = line_arc(
            DVec2::new(-10.0, 0.0), DVec2::new(10.0, 0.0),
            DVec2::new(0.0, 0.0), 5.0,
            0.0, std::f64::consts::PI,
        );
        // Line y=0 crosses circle at (-5,0) = 180° and (5,0) = 0°
        // Since arc is 0°→180°, both should be included (180° is the end).
        assert_eq!(pts.len(), 2, "Expected 2 intersections, got {}", pts.len());
    }
}
