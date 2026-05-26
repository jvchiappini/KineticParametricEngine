use glam::DVec3;
use crate::predicates::EPSILON;

/// Split `tri` with the infinite line that passes through `seg[0]` and `seg[1]`,
/// lying in the plane defined by `normal`.
///
/// Returns up to 3 sub-triangles that tile the original, all with the same
/// winding order as the input.  Returns `vec![tri]` unchanged if the segment
/// does not properly cross the triangle.
pub fn split_triangle_with_normal(
    tri: [DVec3; 3],
    seg: [DVec3; 2],
    normal: DVec3,
) -> Vec<[DVec3; 3]> {
    let p = seg[0];
    let q = seg[1];

    if (q - p).length_squared() < EPSILON * EPSILON {
        return vec![tri];
    }

    // In-plane normal of the split line: perpendicular to q−p within the triangle plane.
    let perp = normal.cross(q - p);
    if perp.length_squared() < EPSILON * EPSILON {
        return vec![tri]; // degenerate segment direction
    }

    // Signed distance of each vertex from the split line.
    let d = [
        perp.dot(tri[0] - p),
        perp.dot(tri[1] - p),
        perp.dot(tri[2] - p),
    ];

    let sign_of = |v: f64| -> i8 {
        if v > EPSILON { 1 } else if v < -EPSILON { -1 } else { 0 }
    };
    let sides = [sign_of(d[0]), sign_of(d[1]), sign_of(d[2])];

    let has_pos = sides.iter().any(|&s| s > 0);
    let has_neg = sides.iter().any(|&s| s < 0);
    if !has_pos || !has_neg {
        return vec![tri]; // all on one side — nothing to split
    }

    // Locate the lone vertex (the one isolated on its own side of the cut).
    let lone_is_pos = sides.iter().filter(|&&s| s > 0).count() == 1;
    let lone = if lone_is_pos {
        (0..3).find(|&i| sides[i] > 0).unwrap()
    } else {
        (0..3).find(|&i| sides[i] < 0).unwrap()
    };
    let next = (lone + 1) % 3;
    let prev = (lone + 2) % 3;

    // Interpolate the two crossing points on the triangle's edges.
    // c0: crossing on edge  lone → next
    let c0 = {
        let denom = d[lone] - d[next];
        if denom.abs() < 1e-30 { return vec![tri]; }
        tri[lone] + (tri[next] - tri[lone]) * (d[lone] / denom)
    };
    // c1: crossing on edge  prev → lone
    let c1 = {
        let denom = d[prev] - d[lone];
        if denom.abs() < 1e-30 { return vec![tri]; }
        tri[prev] + (tri[lone] - tri[prev]) * (d[prev] / denom)
    };

    if (c0 - c1).length_squared() < EPSILON * EPSILON {
        return vec![tri];
    }

    // Three sub-triangles that partition the original and preserve winding:
    //
    //   lone-side (tip):      [tri[lone], c0,         c1          ]
    //   quad first half:      [c0,        tri[next],  tri[prev]   ]
    //   quad second half:     [c0,        tri[prev],  c1          ]
    //
    // Verified correct CCW ordering by cross-product for arbitrary lone index.
    let candidates = [
        [tri[lone], c0, c1],
        [c0, tri[next], tri[prev]],
        [c0, tri[prev], c1],
    ];

    // Drop any degenerate slivers.
    candidates.into_iter().filter(|t| {
        (t[1] - t[0]).cross(t[2] - t[0]).length_squared() > EPSILON * EPSILON
    }).collect()
}

pub fn split_mesh_triangles(
    vertices: &[DVec3],
    triangles: &[[u32; 3]],
    pairs: &[(usize, usize)],
    other_vertices: &[DVec3],
    other_triangles: &[[u32; 3]],
) -> (Vec<DVec3>, Vec<[u32; 3]>) {
    use std::collections::HashMap;
    use crate::intersection::{triangle_triangle_intersection, TriTriIntersection};

    let mut split_tris: Vec<Option<Vec<[u32; 3]>>> = vec![None; triangles.len()];
    let mut new_verts: Vec<DVec3> = vertices.to_vec();

    let mut pairs_by_a: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(ai, bi) in pairs {
        pairs_by_a.entry(ai).or_default().push(bi);
    }

    for (ai, bis) in pairs_by_a {
        let t_a = [
            vertices[triangles[ai][0] as usize],
            vertices[triangles[ai][1] as usize],
            vertices[triangles[ai][2] as usize],
        ];

        let normal = crate::predicates::triangle_normal(t_a[0], t_a[1], t_a[2]);
        let mut fragments = vec![t_a];

        for bi in bis {
            let t_b = [
                other_vertices[other_triangles[bi][0] as usize],
                other_vertices[other_triangles[bi][1] as usize],
                other_vertices[other_triangles[bi][2] as usize],
            ];

            let mut next_fragments = Vec::new();
            for frag in fragments {
                // Use the intersection segment to define where to cut `frag`.
                // No need to sort seg endpoints — the split is defined by the
                // infinite line through both points so order is irrelevant.
                if let TriTriIntersection::Segment(seg) =
                    triangle_triangle_intersection(frag, t_b)
                {
                    let new_frags = split_triangle_with_normal(frag, seg, normal);
                    next_fragments.extend(new_frags);
                } else {
                    next_fragments.push(frag);
                }
            }
            fragments = next_fragments;
        }

        if fragments.len() > 1 {
            let mut frag_indices = Vec::new();
            for frag in &fragments {
                let mut tri_idx = Vec::new();
                for &v in frag {
                    let found = new_verts
                        .iter()
                        .position(|nv| (*nv - v).length_squared() < EPSILON * EPSILON);
                    let idx = if let Some(pos) = found {
                        pos as u32
                    } else {
                        let pos = new_verts.len();
                        new_verts.push(v);
                        pos as u32
                    };
                    tri_idx.push(idx);
                }
                if tri_idx.len() == 3 {
                    frag_indices.push([tri_idx[0], tri_idx[1], tri_idx[2]]);
                }
            }
            split_tris[ai] = Some(frag_indices);
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
