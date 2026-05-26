use glam::DVec3;
use crate::predicates::{self, Sign, EPSILON};

#[derive(Debug, Clone)]
pub enum TriTriIntersection {
    None,
    Coplanar,
    Segment([DVec3; 2]),
}

fn signed_dist(p: DVec3, normal: DVec3, d: f64) -> f64 {
    normal.dot(p) + d
}

fn edge_plane_intersect(a: DVec3, b: DVec3, normal: DVec3, d: f64) -> Option<DVec3> {
    let da = signed_dist(a, normal, d);
    let db = signed_dist(b, normal, d);
    if da.abs() < EPSILON && db.abs() < EPSILON {
        return None;
    }
    if da.signum() == db.signum() {
        return None;
    }
    let t = da.abs() / (da.abs() + db.abs());
    Some(a + (b - a) * t)
}

fn point_in_triangle_2d(p: DVec3, a: DVec3, b: DVec3, c: DVec3, normal: DVec3) -> bool {
    let u = if normal.x.abs() > normal.y.abs() {
        if normal.x.abs() > normal.z.abs() { (1, 2) } else { (0, 1) }
    } else {
        if normal.y.abs() > normal.z.abs() { (0, 2) } else { (0, 1) }
    };

    let p2 = [p.x, p.y, p.z];
    let a2 = [a.x, a.y, a.z];
    let b2 = [b.x, b.y, b.z];
    let c2 = [c.x, c.y, c.z];

    let (u0, u1) = u;
    
    let p_coord = robust::Coord { x: p2[u0], y: p2[u1] };
    let a_coord = robust::Coord { x: a2[u0], y: a2[u1] };
    let b_coord = robust::Coord { x: b2[u0], y: b2[u1] };
    let c_coord = robust::Coord { x: c2[u0], y: c2[u1] };

    let e0 = robust::orient2d(a_coord, b_coord, p_coord);
    let e1 = robust::orient2d(b_coord, c_coord, p_coord);
    let e2 = robust::orient2d(c_coord, a_coord, p_coord);

    let has_neg = e0 < 0.0 || e1 < 0.0 || e2 < 0.0;
    let has_pos = e0 > 0.0 || e1 > 0.0 || e2 > 0.0;

    !(has_neg && has_pos)
}

fn clip_segment_to_triangle(
    p1: DVec3, p2: DVec3,
    ta: DVec3, tb: DVec3, tc: DVec3,
    normal: DVec3,
) -> Option<[DVec3; 2]> {
    if point_in_triangle_2d(p1, ta, tb, tc, normal) && point_in_triangle_2d(p2, ta, tb, tc, normal) {
        return Some([p1, p2]);
    }
    if !point_in_triangle_2d(p1, ta, tb, tc, normal) && !point_in_triangle_2d(p2, ta, tb, tc, normal) {
        return None;
    }

    let edges = [(ta, tb), (tb, tc), (tc, ta)];
    let mut clip_points = Vec::new();
    let dir = (p2 - p1).normalize();

    if point_in_triangle_2d(p1, ta, tb, tc, normal) {
        clip_points.push(p1);
    }
    if point_in_triangle_2d(p2, ta, tb, tc, normal) {
        clip_points.push(p2);
    }

    for &(ea, eb) in &edges {
        let e_normal = normal.cross(eb - ea).normalize();
        let e_d = -e_normal.dot(ea);
        if let Some(p) = edge_plane_intersect(p1, p2, e_normal, e_d) {
            let d1 = (p - ea).cross(eb - ea).dot(normal);
            if d1.abs() < EPSILON && (p - ea).dot(eb - ea) >= -EPSILON && (p - eb).dot(ea - eb) >= -EPSILON {
                clip_points.push(p);
            }
        }
    }

    if clip_points.len() < 2 {
        return None;
    }

    clip_points.sort_by(|a, b| {
        dir.dot(*a - p1).partial_cmp(&dir.dot(*b - p1)).unwrap()
    });

    Some([clip_points[0], clip_points[clip_points.len() - 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_intersection_separated() {
        let t1 = [DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 1.0, 0.0)];
        let t2 = [DVec3::new(0.0, 0.0, 5.0), DVec3::new(1.0, 0.0, 5.0), DVec3::new(0.0, 1.0, 5.0)];
        let result = triangle_triangle_intersection(t1, t2);
        assert!(matches!(result, TriTriIntersection::None));
    }

    #[test]
    fn test_intersection_coplanar() {
        let t1 = [DVec3::new(0.0, 0.0, 0.0), DVec3::new(2.0, 0.0, 0.0), DVec3::new(0.0, 2.0, 0.0)];
        let t2 = [DVec3::new(1.0, 0.0, 0.0), DVec3::new(3.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 0.0)];
        let result = triangle_triangle_intersection(t1, t2);
        assert!(matches!(result, TriTriIntersection::Coplanar | TriTriIntersection::None));
    }

    #[test]
    fn test_intersection_segment() {
        let t1 = [DVec3::new(0.0, 0.0, 0.0), DVec3::new(2.0, 0.0, 0.0), DVec3::new(0.0, 2.0, 0.0)];
        let t2 = [DVec3::new(0.5, 0.5, -1.0), DVec3::new(0.5, 0.5, 1.0), DVec3::new(1.5, 0.5, 0.0)];
        let result = triangle_triangle_intersection(t1, t2);
        match result {
            TriTriIntersection::Segment(seg) => {
                assert!((seg[0] - seg[1]).length() > 0.0);
            }
            other => {
                panic!("Expected Segment, got {:?}", other);
            }
        }
    }

    #[test]
    fn test_touching_at_edge() {
        let t1 = [DVec3::new(0.0, 0.0, 0.0), DVec3::new(1.0, 0.0, 0.0), DVec3::new(0.0, 1.0, 0.0)];
        let t2 = [DVec3::new(1.0, 0.0, 0.0), DVec3::new(2.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 0.0)];
        let result = triangle_triangle_intersection(t1, t2);
        assert!(matches!(result, TriTriIntersection::Coplanar | TriTriIntersection::None));
    }
}

pub fn triangle_triangle_intersection(
    t1: [DVec3; 3],
    t2: [DVec3; 3],
) -> TriTriIntersection {
    let n1 = predicates::triangle_normal(t1[0], t1[1], t1[2]);
    let d1 = -n1.dot(t1[0]);
    let n2 = predicates::triangle_normal(t2[0], t2[1], t2[2]);
    let d2 = -n2.dot(t2[0]);

    let d2_vals = [
        signed_dist(t2[0], n1, d1),
        signed_dist(t2[1], n1, d1),
        signed_dist(t2[2], n1, d1),
    ];

    let s2 = [
        predicates::sign(d2_vals[0]),
        predicates::sign(d2_vals[1]),
        predicates::sign(d2_vals[2]),
    ];

    if s2[0] != Sign::Zero && s2[0] == s2[1] && s2[1] == s2[2] {
        return TriTriIntersection::None;
    }

    let mut intersect_pts = Vec::new();
    let edges2 = [(0, 1), (1, 2), (2, 0)];
    for &(i, j) in &edges2 {
        if s2[i] != s2[j] || s2[i] == Sign::Zero {
            if let Some(p) = edge_plane_intersect(t2[i], t2[j], n1, d1) {
                intersect_pts.push(p);
            }
        }
    }

    if intersect_pts.is_empty() {
        let d1_vals = [
            signed_dist(t1[0], n2, d2),
            signed_dist(t1[1], n2, d2),
            signed_dist(t1[2], n2, d2),
        ];
        let s1 = [
            predicates::sign(d1_vals[0]),
            predicates::sign(d1_vals[1]),
            predicates::sign(d1_vals[2]),
        ];
        if s1[0] == Sign::Zero && s1[1] == Sign::Zero && s1[2] == Sign::Zero {
            return TriTriIntersection::Coplanar;
        }
        return TriTriIntersection::None;
    }

    if intersect_pts.len() == 1 {
        let d1_vals = [
            signed_dist(t1[0], n2, d2),
            signed_dist(t1[1], n2, d2),
            signed_dist(t1[2], n2, d2),
        ];
        let s1 = [
            predicates::sign(d1_vals[0]),
            predicates::sign(d1_vals[1]),
            predicates::sign(d1_vals[2]),
        ];
        let edges1 = [(0, 1), (1, 2), (2, 0)];
        for &(i, j) in &edges1 {
            if s1[i] != s1[j] || s1[i] == Sign::Zero {
                if let Some(p) = edge_plane_intersect(t1[i], t1[j], n2, d2) {
                    if (p - intersect_pts[0]).length_squared() > EPSILON * EPSILON {
                        intersect_pts.push(p);
                        break;
                    }
                }
            }
        }
    }

    if intersect_pts.len() < 2 {
        return TriTriIntersection::None;
    }

    let seg = [intersect_pts[0], intersect_pts[1]];

    if (seg[0] - seg[1]).length_squared() < EPSILON * EPSILON {
        return TriTriIntersection::None;
    }

    if let Some(clipped) = clip_segment_to_triangle(seg[0], seg[1], t2[0], t2[1], t2[2], n2) {
        if let Some(clipped2) = clip_segment_to_triangle(clipped[0], clipped[1], t1[0], t1[1], t1[2], n1) {
            if (clipped2[0] - clipped2[1]).length_squared() > EPSILON * EPSILON {
                return TriTriIntersection::Segment(clipped2);
            }
        }
    }

    TriTriIntersection::None
}
