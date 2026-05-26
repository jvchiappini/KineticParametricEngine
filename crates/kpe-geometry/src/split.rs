use glam::DVec3;
use crate::predicates::{self, EPSILON};

fn orient2d_bare(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> f64 {
    (ax - cx) * (by - cy) - (bx - cx) * (ay - cy)
}

fn project_2d(v: DVec3, normal: DVec3) -> (f64, f64) {
    let u = if normal.x.abs() > normal.y.abs() {
        if normal.x.abs() > normal.z.abs() { (1, 2) } else { (0, 1) }
    } else {
        if normal.y.abs() > normal.z.abs() { (0, 2) } else { (0, 1) }
    };
    match u {
        (0, 1) => (v.x, v.y),
        (1, 2) => (v.y, v.z),
        (0, 2) => (v.x, v.z),
        _ => (v.x, v.y),
    }
}

fn orient_sign_2d(p: (f64, f64), q: (f64, f64), r: (f64, f64)) -> i8 {
    let o = orient2d_bare(p.0, p.1, q.0, q.1, r.0, r.1);
    if o.abs() < 1e-9 { 0 } else if o > 0.0 { 1 } else { -1 }
}

pub fn split_triangle(
    tri: [DVec3; 3],
    seg: [DVec3; 2],
) -> Vec<[DVec3; 3]> {
    let p = seg[0];
    let q = seg[1];

    if (p - q).length_squared() < EPSILON * EPSILON {
        return vec![tri];
    }

    let normal = predicates::triangle_normal(tri[0], tri[1], tri[2]);

    let t0 = project_2d(tri[0], normal);
    let t1 = project_2d(tri[1], normal);
    let t2 = project_2d(tri[2], normal);
    let pp = project_2d(p, normal);
    let pq = project_2d(q, normal);

    let sides = [
        orient_sign_2d(pp, pq, t0),
        orient_sign_2d(pp, pq, t1),
        orient_sign_2d(pp, pq, t2),
    ];

    let pos: Vec<usize> = sides.iter().enumerate().filter(|(_, &s)| s == 1).map(|(i, _)| i).collect();
    let neg: Vec<usize> = sides.iter().enumerate().filter(|(_, &s)| s == -1).map(|(i, _)| i).collect();

    if pos.is_empty() || neg.is_empty() {
        return vec![tri];
    }

    let mut fragments = Vec::new();

    if pos.len() == 1 && neg.len() == 2 {
        let lone = pos[0];
        let p0 = neg[0];
        let p1 = neg[1];

        fragments.push([tri[lone], p, q]);

        fragments.push([tri[p0], tri[p1], q]);
        fragments.push([tri[p0], q, p]);
    } else if neg.len() == 1 && pos.len() == 2 {
        let lone = neg[0];
        let p0 = pos[0];
        let p1 = pos[1];

        fragments.push([tri[lone], q, p]);

        fragments.push([tri[p0], tri[p1], p]);
        fragments.push([tri[p0], p, q]);
    } else {
        return vec![tri];
    }

    fragments.retain(|tri| {
        let a = tri[0]; let b = tri[1]; let c = tri[2];
        let e01 = (a - b).length_squared();
        let e12 = (b - c).length_squared();
        let e20 = (c - a).length_squared();
        e01 > EPSILON && e12 > EPSILON && e20 > EPSILON
            && (b - a).cross(c - a).length_squared() > EPSILON * EPSILON
    });

    fragments
}

pub fn split_mesh_triangles(
    vertices: &[DVec3],
    triangles: &[[u32; 3]],
    pairs: &[(usize, usize)],
    other_vertices: &[DVec3],
    other_triangles: &[[u32; 3]],
) -> (Vec<DVec3>, Vec<[u32; 3]>) {
    use std::collections::HashSet;
    use crate::intersection::{triangle_triangle_intersection, TriTriIntersection};

    let mut split_tris: Vec<Option<Vec<[u32; 3]>>> = vec![None; triangles.len()];
    let mut new_verts: Vec<DVec3> = vertices.to_vec();
    let mut processed: HashSet<usize> = HashSet::new();

    for &(ai, bi) in pairs {
        if processed.contains(&ai) {
            continue;
        }
        processed.insert(ai);

        let t_a = [
            vertices[triangles[ai][0] as usize],
            vertices[triangles[ai][1] as usize],
            vertices[triangles[ai][2] as usize],
        ];
        let t_b = [
            other_vertices[other_triangles[bi][0] as usize],
            other_vertices[other_triangles[bi][1] as usize],
            other_vertices[other_triangles[bi][2] as usize],
        ];

        if let TriTriIntersection::Segment(mut seg) = triangle_triangle_intersection(t_a, t_b) {
            seg.sort_by(|a, b| {
                a.x.partial_cmp(&b.x).unwrap()
                    .then(a.y.partial_cmp(&b.y).unwrap())
                    .then(a.z.partial_cmp(&b.z).unwrap())
            });

            let fragments = split_triangle(t_a, seg);
            if fragments.len() > 1 {
                let mut frag_indices = Vec::new();
                for frag in &fragments {
                    let mut tri_idx = Vec::new();
                    for &v in frag {
                        let found = new_verts.iter()
                            .position(|nv| (*nv - v).length_squared() < EPSILON * EPSILON);
                        if let Some(pos) = found {
                            tri_idx.push(pos as u32);
                        } else {
                            let pos = new_verts.len();
                            new_verts.push(v);
                            tri_idx.push(pos as u32);
                        }
                    }
                    if tri_idx.len() == 3 {
                        frag_indices.push([tri_idx[0], tri_idx[1], tri_idx[2]]);
                    }
                }
                split_tris[ai] = Some(frag_indices);
            }
        }
    }

    let mut result_tris = Vec::new();
    for (i, tri) in triangles.iter().enumerate() {
        if let Some(ref fragments) = split_tris[i] {
            result_tris.extend_from_slice(fragments);
        } else {
            result_tris.push(*tri);
        }
    }

    (new_verts, result_tris)
}
