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

pub fn classify_point_against_mesh(
    point: DVec3,
    vertices: &[DVec3],
    triangles: &[[u32; 3]],
) -> bool {
    let mut winding_number = 0i32;
    let ray_dir = DVec3::X;

    for tri in triangles {
        let a = vertices[tri[0] as usize];
        let b = vertices[tri[1] as usize];
        let c = vertices[tri[2] as usize];

        let normal = predicates::triangle_normal(a, b, c);

        let bc = match barycentric(point, a, b, c) {
            Some(bc) => bc,
            None => continue,
        };

        if bc.u >= -EPSILON && bc.v >= -EPSILON && bc.w >= -EPSILON {
            let dist = (point - (a * bc.u + b * bc.v + c * bc.w)).length();
            if dist < EPSILON * 10.0 {
                return true;
            }
        }

        if normal.dot(ray_dir).abs() < EPSILON {
            continue;
        }

        let d = -(normal.dot(a));
        let t = -(normal.dot(point) + d) / normal.dot(ray_dir);

        if t < EPSILON {
            continue;
        }

        let hit = point + ray_dir * t;

        let bc2 = match barycentric(hit, a, b, c) {
            Some(bc2) => bc2,
            None => continue,
        };

        let eps = 1e-9;
        if bc2.u >= -eps && bc2.v >= -eps && bc2.w >= -eps {
            if normal.dot(ray_dir) > 0.0 {
                winding_number += 1;
            } else {
                winding_number -= 1;
            }
        }
    }

    winding_number != 0
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
