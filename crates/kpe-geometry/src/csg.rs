use glam::DVec3;
use kpe_schema::geometry::{CsgOpType, CsgOperation, TriangleMesh, BRepModel};
use crate::bvh::BVH;
use crate::classify::classify_triangle_fragments;
use crate::stitch::Stitcher;


pub struct CsgKernel;

fn to_dvec3(mesh: &TriangleMesh) -> (Vec<DVec3>, Vec<[u32; 3]>) {
    let verts: Vec<DVec3> = mesh.vertices.iter().map(|v| DVec3::new(v[0], v[1], v[2])).collect();
    (verts, mesh.triangles.clone())
}

fn from_dvec3(verts: &[DVec3], tris: &[[u32; 3]]) -> TriangleMesh {
    let vertices: Vec<[f64; 3]> = verts.iter().map(|v| [v.x, v.y, v.z]).collect();
    TriangleMesh {
        vertices,
        normals: vec![],
        uvs: vec![],
        triangles: tris.to_vec(),
    }
}

impl CsgKernel {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_operation(
        &self,
        mesh_a: &TriangleMesh,
        mesh_b: &TriangleMesh,
        operation: &CsgOperation,
    ) -> TriangleMesh {
        match operation.op_type {
            CsgOpType::Union => self.union(mesh_a, mesh_b),
            CsgOpType::Subtract => self.subtract(mesh_a, mesh_b),
            CsgOpType::Intersect => self.intersect(mesh_a, mesh_b),
        }
    }

    pub fn union(&self, mesh_a: &TriangleMesh, mesh_b: &TriangleMesh) -> TriangleMesh {
        let (va, ta) = to_dvec3(mesh_a);
        let (vb, tb) = to_dvec3(mesh_b);

        let bvh_a = BVH::build(&va, &ta);
        let bvh_b = BVH::build(&vb, &tb);
        let pairs = bvh_a.query_intersections(&bvh_b);

        if pairs.is_empty() {
            return self.concatenate(mesh_a, mesh_b);
        }

        let a_class = classify_triangle_fragments(&va, &ta, &vb, &tb);
        let b_class = classify_triangle_fragments(&vb, &tb, &va, &ta);

        let mut out_verts: Vec<DVec3> = Vec::new();
        let mut out_tris: Vec<[u32; 3]> = Vec::new();

        for (i, tri) in ta.iter().enumerate() {
            if !a_class[i] {
                let base = out_verts.len() as u32;
                out_verts.push(va[tri[0] as usize]);
                out_verts.push(va[tri[1] as usize]);
                out_verts.push(va[tri[2] as usize]);
                out_tris.push([base, base + 1, base + 2]);
            }
        }

        for (i, tri) in tb.iter().enumerate() {
            if !b_class[i] {
                let base = out_verts.len() as u32;
                out_verts.push(vb[tri[0] as usize]);
                out_verts.push(vb[tri[1] as usize]);
                out_verts.push(vb[tri[2] as usize]);
                out_tris.push([base, base + 1, base + 2]);
            }
        }

        let stitcher = Stitcher::new();
        let (sv, st) = stitcher.stitch(&out_verts, &out_tris);
        from_dvec3(&sv, &st)
    }

    pub fn subtract(&self, mesh_a: &TriangleMesh, mesh_b: &TriangleMesh) -> TriangleMesh {
        let (va, ta) = to_dvec3(mesh_a);
        let (vb, tb) = to_dvec3(mesh_b);

        let bvh_a = BVH::build(&va, &ta);
        let bvh_b = BVH::build(&vb, &tb);
        let pairs = bvh_a.query_intersections(&bvh_b);

        if pairs.is_empty() {
            return mesh_a.clone();
        }

        let a_class = classify_triangle_fragments(&va, &ta, &vb, &tb);
        let b_class = classify_triangle_fragments(&vb, &tb, &va, &ta);

        let mut out_verts: Vec<DVec3> = Vec::new();
        let mut out_tris: Vec<[u32; 3]> = Vec::new();

        for (i, tri) in ta.iter().enumerate() {
            if !a_class[i] {
                let base = out_verts.len() as u32;
                out_verts.push(va[tri[0] as usize]);
                out_verts.push(va[tri[1] as usize]);
                out_verts.push(va[tri[2] as usize]);
                out_tris.push([base, base + 1, base + 2]);
            }
        }

        for (i, tri) in tb.iter().enumerate() {
            if b_class[i] {
                let a = vb[tri[0] as usize];
                let b = vb[tri[1] as usize];
                let c = vb[tri[2] as usize];
                let base = out_verts.len() as u32;
                out_verts.push(c);
                out_verts.push(b);
                out_verts.push(a);
                out_tris.push([base, base + 1, base + 2]);
            }
        }

        let stitcher = Stitcher::new();
        let (sv, st) = stitcher.stitch(&out_verts, &out_tris);
        from_dvec3(&sv, &st)
    }

    pub fn intersect(&self, mesh_a: &TriangleMesh, mesh_b: &TriangleMesh) -> TriangleMesh {
        let (va, ta) = to_dvec3(mesh_a);
        let (vb, tb) = to_dvec3(mesh_b);

        let bvh_a = BVH::build(&va, &ta);
        let bvh_b = BVH::build(&vb, &tb);
        let pairs = bvh_a.query_intersections(&bvh_b);

        if pairs.is_empty() {
            return TriangleMesh {
                vertices: vec![],
                normals: vec![],
                uvs: vec![],
                triangles: vec![],
            };
        }

        let a_class = classify_triangle_fragments(&va, &ta, &vb, &tb);
        let b_class = classify_triangle_fragments(&vb, &tb, &va, &ta);

        let mut out_verts: Vec<DVec3> = Vec::new();
        let mut out_tris: Vec<[u32; 3]> = Vec::new();

        for (i, tri) in ta.iter().enumerate() {
            if a_class[i] {
                let base = out_verts.len() as u32;
                out_verts.push(va[tri[0] as usize]);
                out_verts.push(va[tri[1] as usize]);
                out_verts.push(va[tri[2] as usize]);
                out_tris.push([base, base + 1, base + 2]);
            }
        }

        for (i, tri) in tb.iter().enumerate() {
            if b_class[i] {
                let base = out_verts.len() as u32;
                out_verts.push(vb[tri[0] as usize]);
                out_verts.push(vb[tri[1] as usize]);
                out_verts.push(vb[tri[2] as usize]);
                out_tris.push([base, base + 1, base + 2]);
            }
        }

        let stitcher = Stitcher::new();
        let (sv, st) = stitcher.stitch(&out_verts, &out_tris);
        from_dvec3(&sv, &st)
    }

    fn concatenate(&self, mesh_a: &TriangleMesh, mesh_b: &TriangleMesh) -> TriangleMesh {
        let mut vertices = mesh_a.vertices.clone();
        let base = vertices.len() as u32;
        vertices.extend_from_slice(&mesh_b.vertices);

        let mut triangles = mesh_a.triangles.clone();
        for tri in &mesh_b.triangles {
            triangles.push([tri[0] + base, tri[1] + base, tri[2] + base]);
        }

        TriangleMesh {
            vertices,
            normals: vec![],
            uvs: vec![],
            triangles,
        }
    }

    pub fn build_brep(&self, mesh: &TriangleMesh) -> BRepModel {
        let mut brep = crate::brep::BRepKernel::new();
        let _vert_ids: Vec<_> = mesh.vertices.iter()
            .map(|v| brep.add_vertex(DVec3::new(v[0], v[1], v[2])))
            .collect();

        let mut face_ids = Vec::new();
        for _ in &mesh.triangles {
            let face_id = brep.add_face(0, None);
            face_ids.push(face_id);
        }

        if !face_ids.is_empty() {
            brep.add_solid(face_ids);
        }

        BRepModel { solids: vec![] }
    }
}

impl Default for CsgKernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::geometry::TriangleMesh;

    fn make_box_at(cx: f64, cy: f64, cz: f64, w: f64, h: f64, d: f64) -> TriangleMesh {
        let hw = w / 2.0;
        let hh = h / 2.0;
        let hd = d / 2.0;
        TriangleMesh {
            vertices: vec![
                [cx - hw, cy - hh, cz - hd], [cx + hw, cy - hh, cz - hd],
                [cx + hw, cy + hh, cz - hd], [cx - hw, cy + hh, cz - hd],
                [cx - hw, cy - hh, cz + hd], [cx + hw, cy - hh, cz + hd],
                [cx + hw, cy + hh, cz + hd], [cx - hw, cy + hh, cz + hd],
            ],
            normals: vec![],
            uvs: vec![],
            triangles: vec![
                [0, 1, 2], [0, 2, 3], [1, 5, 6], [1, 6, 2],
                [5, 4, 7], [5, 7, 6], [4, 0, 3], [4, 3, 7],
                [3, 2, 6], [3, 6, 7], [4, 5, 1], [4, 1, 0],
            ],
        }
    }

    #[test]
    fn test_union_non_overlapping() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mesh_b = make_box_at(5.0, 5.0, 5.0, 1.0, 1.0, 1.0);
        let result = kernel.union(&mesh_a, &mesh_b);
        let expected_count = mesh_a.triangles.len() + mesh_b.triangles.len();
        assert_eq!(result.triangles.len(), expected_count);
    }

    #[test]
    fn test_subtract_non_overlapping() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mesh_b = make_box_at(10.0, 10.0, 10.0, 1.0, 1.0, 1.0);
        let result = kernel.subtract(&mesh_a, &mesh_b);
        assert_eq!(result.triangles.len(), mesh_a.triangles.len());
    }

    #[test]
    fn test_intersect_non_overlapping() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mesh_b = make_box_at(10.0, 10.0, 10.0, 1.0, 1.0, 1.0);
        let result = kernel.intersect(&mesh_a, &mesh_b);
        assert!(result.triangles.is_empty());
    }

    #[test]
    fn test_union_adjacent_boxes() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 2.0, 1.0, 1.0);
        let mesh_b = make_box_at(2.0, 0.0, 0.0, 2.0, 1.0, 1.0);
        let result = kernel.union(&mesh_a, &mesh_b);
        let joint = mesh_a.triangles.len() + mesh_b.triangles.len();
        assert!(result.triangles.len() <= joint);
        assert!(result.triangles.len() >= mesh_a.triangles.len());
        assert!(!result.triangles.is_empty());
    }

    #[test]
    fn test_subtract_inner_removes_triangles() {
        let kernel = CsgKernel::new();
        let outer = make_box_at(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
        let inner = make_box_at(0.0, 0.0, 0.0, 4.0, 4.0, 4.0);
        let result = kernel.subtract(&outer, &inner);
        assert!(result.triangles.len() < outer.triangles.len() + inner.triangles.len());
        assert!(!result.triangles.is_empty());
    }

    #[test]
    fn test_subtract_fully_inside() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
        let mesh_b = make_box_at(0.0, 0.0, 0.0, 2.0, 2.0, 2.0);
        let result = kernel.subtract(&mesh_a, &mesh_b);
        assert!(!result.triangles.is_empty());
    }
}
