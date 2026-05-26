use std::collections::HashMap;
use glam::DVec3;
use crate::predicates::EPSILON;

pub struct Stitcher;

impl Stitcher {
    pub fn new() -> Self {
        Self
    }

    pub fn stitch(
        &self,
        vertices: &[DVec3],
        triangles: &[[u32; 3]],
    ) -> (Vec<DVec3>, Vec<[u32; 3]>) {
        let (merged_verts, vert_map) = self.merge_vertices(vertices);

        let mut out_triangles = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for tri in triangles {
            let t0 = vert_map[&tri[0]];
            let t1 = vert_map[&tri[1]];
            let t2 = vert_map[&tri[2]];

            if t0 == t1 || t1 == t2 || t2 == t0 {
                continue;
            }

            let a = merged_verts[t0 as usize];
            let b = merged_verts[t1 as usize];
            let c = merged_verts[t2 as usize];
            let area = (b - a).cross(c - a).length();
            if area < EPSILON {
                continue;
            }

            // Canonicalise by cyclic rotation (find the minimum-index position and
            // rotate to that position), preserving winding order so that [0,1,2] and
            // [0,2,1] are treated as *distinct* triangles (opposite normals).
            let min_pos = if t0 <= t1 && t0 <= t2 { 0 } else if t1 <= t2 { 1 } else { 2 };
            let key = match min_pos {
                0 => [t0, t1, t2],
                1 => [t1, t2, t0],
                _ => [t2, t0, t1],
            };
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);

            out_triangles.push([t0, t1, t2]);
        }

        (merged_verts, out_triangles)
    }

    fn merge_vertices(&self, vertices: &[DVec3]) -> (Vec<DVec3>, HashMap<u32, u32>) {
        let mut merged = Vec::new();
        let mut map = HashMap::new();

        for (i, v) in vertices.iter().enumerate() {
            let mut found = false;
            for (j, mv) in merged.iter().enumerate() {
                let diff: DVec3 = *v - *mv;
                if diff.length_squared() < EPSILON * EPSILON {
                    map.insert(i as u32, j as u32);
                    found = true;
                    break;
                }
            }
            if !found {
                let idx = merged.len();
                merged.push(*v);
                map.insert(i as u32, idx as u32);
            }
        }

        (merged, map)
    }
}

impl Default for Stitcher {
    fn default() -> Self {
        Self::new()
    }
}
