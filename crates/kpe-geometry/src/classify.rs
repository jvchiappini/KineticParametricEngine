use glam::DVec3;
use crate::predicates::{self, EPSILON};

struct BarycentricCoords {
    u: f64,
    v: f64,
    w: f64,
}

fn barycentric(p: DVec3, a: DVec3, b: DVec3, c: DVec3) -> Option<BarycentricCoords> {
    let v0 = c - a;
    let v1 = b - a;
    let v2 = p - a;
    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);
    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < EPSILON {
        return None;
    }
    let inv = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv;
    let v = (dot00 * dot12 - dot01 * dot02) * inv;
    let w = 1.0 - u - v;
    Some(BarycentricCoords { u, v, w })
}

/// Cast a single ray from `point` in `ray_dir` and return the winding-number
/// contribution from `triangles`. Returns `None` if the ray is degenerate
/// (grazes a triangle edge/vertex exactly) for this direction.
fn winding_for_ray(
    point: DVec3,
    ray_dir: DVec3,
    vertices: &[DVec3],
    triangles: &[[u32; 3]],
) -> Option<i32> {
    let mut winding = 0i32;

    for tri in triangles {
        let a = vertices[tri[0] as usize];
        let b = vertices[tri[1] as usize];
        let c = vertices[tri[2] as usize];

        let normal = predicates::triangle_normal(a, b, c);
        let dot = normal.dot(ray_dir);

        if dot.abs() < EPSILON {
            continue;
        }

        let d = -(normal.dot(a));
        let t = -(normal.dot(point) + d) / dot;

        if t < EPSILON {
            continue;
        }

        let hit = point + ray_dir * t;

        let bc = match barycentric(hit, a, b, c) {
            Some(bc) => bc,
            None => continue,
        };

        // If the hit lands exactly on an edge or vertex, this ray direction is
        // degenerate — signal the caller to retry with a different direction.
        let on_edge = bc.u.abs() < 1e-9 || bc.v.abs() < 1e-9 || bc.w.abs() < 1e-9;
        if on_edge {
            return None;
        }

        if bc.u >= 0.0 && bc.v >= 0.0 && bc.w >= 0.0 {
            if dot > 0.0 {
                winding += 1;
            } else {
                winding -= 1;
            }
        }
    }

    Some(winding)
}

pub fn classify_point_against_mesh(
    point: DVec3,
    vertices: &[DVec3],
    triangles: &[[u32; 3]],
) -> bool {
    // Use 3 ray directions with irrational components to avoid axis-aligned degeneracies.
    // Vote: a point is inside if the majority of non-degenerate rays agree.
    let ray_dirs = [
        DVec3::new(1.0, 0.7071067811865476, 0.5773502691896258).normalize(), // ~(1, √½, 1/√3)
        DVec3::new(-0.5773502691896258, 1.0, 0.7071067811865476).normalize(),
        DVec3::new(0.7071067811865476, -0.5773502691896258, 1.0).normalize(),
    ];

    let mut votes_inside = 0i32;
    let mut votes_outside = 0i32;

    for ray_dir in ray_dirs {
        if let Some(w) = winding_for_ray(point, ray_dir, vertices, triangles) {
            if w != 0 {
                votes_inside += 1;
            } else {
                votes_outside += 1;
            }
        }
        // Degenerate ray: skip — the other two will still decide.
    }

    votes_inside > votes_outside
}

pub fn classify_triangle_fragments(
    vertices: &[DVec3],
    triangles: &[[u32; 3]],
    other_vertices: &[DVec3],
    other_triangles: &[[u32; 3]],
) -> Vec<bool> {
    let mut inside = Vec::with_capacity(triangles.len());
    for tri in triangles {
        let center = (vertices[tri[0] as usize]
            + vertices[tri[1] as usize]
            + vertices[tri[2] as usize]) / 3.0;
        let is_inside = classify_point_against_mesh(center, other_vertices, other_triangles);
        inside.push(is_inside);
    }
    inside
}
