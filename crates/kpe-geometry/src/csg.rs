use kpe_schema::geometry::{CsgOpType, CsgOperation, TriangleMesh, BRepModel};
use glam::DVec3;

pub struct CsgKernel;

// ── Manifold backend (feature = "manifold") ──────────────────────────────────

#[cfg(feature = "manifold")]
mod manifold_backend {
    use manifold_csg::Manifold;
    use kpe_schema::geometry::{CsgOpType, TriangleMesh};

    /// Convert KPE `TriangleMesh` → `Manifold`.
    ///
    /// `from_mesh_f64` expects:
    ///   - `vert_props`: flat f64 array [x0,y0,z0, x1,y1,z1, ...]
    ///   - `n_props`: properties per vertex (3 = xyz only)
    ///   - `tri_indices`: flat u64 array [v0,v1,v2, ...]
    pub fn to_manifold(mesh: &TriangleMesh) -> Option<Manifold> {
        if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
            return None;
        }

        // Flat vertex array: [x, y, z, x, y, z, ...]
        let vert_props: Vec<f64> = mesh.vertices.iter().flat_map(|v| [v[0], v[1], v[2]]).collect();

        // Flat triangle index array as u64
        let tri_indices: Vec<u64> = mesh
            .triangles
            .iter()
            .flat_map(|t| [t[0] as u64, t[1] as u64, t[2] as u64])
            .collect();

        match Manifold::from_mesh_f64(&vert_props, 3, &tri_indices) {
            Ok(m) => Some(m),
            Err(e) => {
                // Log but don't panic — we fall back to empty rather than crash
                eprintln!("[manifold] from_mesh_f64 failed: {:?}", e);
                None
            }
        }
    }

    /// Convert `Manifold` → KPE `TriangleMesh`.
    ///
    /// `to_mesh_f64_with_normals(3)` returns vert_props with layout
    /// [x, y, z, nx, ny, nz] per vertex and n_props = 6.
    pub fn from_manifold(m: &Manifold) -> TriangleMesh {
        // normal_idx = 3 → normals start at slot 3 in the property array
        let (vert_props, n_props, tri_indices) = m.to_mesh_f64_with_normals(3);

        if vert_props.is_empty() || tri_indices.is_empty() {
            return empty_mesh();
        }

        let num_verts = vert_props.len() / n_props;

        let mut vertices = Vec::with_capacity(num_verts);
        let mut normals  = Vec::with_capacity(num_verts);

        for chunk in vert_props.chunks_exact(n_props) {
            vertices.push([chunk[0], chunk[1], chunk[2]]);
            // normals are at indices 3,4,5 (present because we passed normal_idx=3)
            if n_props >= 6 {
                normals.push([chunk[3], chunk[4], chunk[5]]);
            } else {
                normals.push([0.0, 0.0, 1.0]);
            }
        }

        let triangles: Vec<[u32; 3]> = tri_indices
            .chunks_exact(3)
            .map(|t| [t[0] as u32, t[1] as u32, t[2] as u32])
            .collect();

        let num_v = vertices.len();
        TriangleMesh {
            vertices,
            normals,
            uvs: vec![[0.0, 0.0]; num_v],
            triangles,
        }
    }

    pub fn empty_mesh() -> TriangleMesh {
        TriangleMesh {
            vertices:  vec![],
            normals:   vec![],
            uvs:       vec![],
            triangles: vec![],
        }
    }

    /// Apply fillet to mesh by rounding sharp edges.
    /// Uses Manifold's smooth_out with radius-based smoothness.
    pub fn apply_fillet(mesh: &TriangleMesh, radius: f64) -> TriangleMesh {
        let m = match to_manifold(mesh) {
            Some(m) => m,
            None => return mesh.clone(),
        };
        let smoothness = (radius / (radius + 0.5)).clamp(0.1, 0.95);
        let result = m.smooth_out(5.0, smoothness);
        from_manifold(&result)
    }

    /// Apply chamfer to mesh by refining and smoothing edges.
    pub fn apply_chamfer(mesh: &TriangleMesh, distance: f64) -> TriangleMesh {
        let m = match to_manifold(mesh) {
            Some(m) => m,
            None => return mesh.clone(),
        };
        let refined = m.refine_to_length(distance * 0.5);
        let result = refined.smooth_out(5.0, 0.3);
        from_manifold(&result)
    }

    pub fn apply_op(
        mesh_a: &TriangleMesh,
        mesh_b: &TriangleMesh,
        op: &CsgOpType,
    ) -> TriangleMesh {
        let a = match to_manifold(mesh_a) {
            Some(m) => m,
            None    => return match op { CsgOpType::Intersect => empty_mesh(), _ => mesh_a.clone() },
        };
        let b = match to_manifold(mesh_b) {
            Some(m) => m,
            None    => return match op { CsgOpType::Intersect => empty_mesh(), _ => mesh_a.clone() },
        };

        let result = match op {
            CsgOpType::Union     => &a + &b,
            CsgOpType::Subtract  => &a - &b,
            CsgOpType::Intersect => &a ^ &b,
        };

        from_manifold(&result)
    }
}

// ── csgrs fallback backend (compiled only when manifold feature is NOT active) ─

#[cfg(not(feature = "manifold"))]
mod csgrs_backend {
    use csgrs::mesh::Mesh;
    use csgrs::mesh::polygon::Polygon;
    use csgrs::mesh::vertex::Vertex;
    use csgrs::traits::CSG;
    use nalgebra::Point3;
    use kpe_schema::geometry::{CsgOpType, TriangleMesh};
    use std::collections::HashMap;

    type CsgMesh = Mesh<()>;

    pub fn empty_mesh() -> TriangleMesh {
        TriangleMesh { vertices: vec![], normals: vec![], uvs: vec![], triangles: vec![] }
    }

    fn triangle_mesh_to_csg(mesh: &TriangleMesh) -> CsgMesh {
        let polygons: Vec<Polygon<()>> = mesh
            .triangles
            .iter()
            .filter_map(|tri| {
                let mut points = [Point3::origin(); 3];
                for i in 0..3 {
                    let p = &mesh.vertices[tri[i] as usize];
                    points[i] = Point3::new(p[0], p[1], p[2]);
                }
                let ab = points[1] - points[0];
                let ac = points[2] - points[0];
                let n  = ab.cross(&ac);
                let len = n.norm();
                let normal = if len > 1e-10 { n / len } else { return None };

                let verts: Vec<Vertex> = [points[0], points[1], points[2]]
                    .iter()
                    .map(|&p| Vertex::new(p, normal))
                    .collect();
                Some(Polygon::new(verts, None))
            })
            .collect();

        Mesh::from_polygons(&polygons, None)
    }

    fn quantize(val: f64) -> i64 { (val * 1e5).round() as i64 }

    fn csg_to_triangle_mesh(csg_mesh: CsgMesh) -> TriangleMesh {
        let mut vertices:  Vec<[f64; 3]> = Vec::new();
        let mut normals:   Vec<[f64; 3]> = Vec::new();
        let mut triangles: Vec<[u32; 3]> = Vec::new();
        let mut vertex_map: HashMap<[i64; 6], u32> = HashMap::new();

        for polygon in &csg_mesh.polygons {
            for tri_verts in polygon.triangulate() {
                let mut indices = [0u32; 3];
                for (i, v) in tri_verts.iter().enumerate() {
                    let bits = [
                        quantize(v.pos.x), quantize(v.pos.y), quantize(v.pos.z),
                        quantize(v.normal.x), quantize(v.normal.y), quantize(v.normal.z),
                    ];
                    if let Some(&idx) = vertex_map.get(&bits) {
                        indices[i] = idx;
                    } else {
                        let idx = vertices.len() as u32;
                        vertices.push([v.pos.x, v.pos.y, v.pos.z]);
                        normals.push([v.normal.x, v.normal.y, v.normal.z]);
                        vertex_map.insert(bits, idx);
                        indices[i] = idx;
                    }
                }
                triangles.push(indices);
            }
        }

        let num_verts = vertices.len();
        TriangleMesh { vertices, normals, uvs: vec![[0.0, 0.0]; num_verts], triangles }
    }

    pub fn apply_fillet(mesh: &TriangleMesh, _radius: f64) -> TriangleMesh {
        mesh.clone()
    }

    pub fn apply_chamfer(mesh: &TriangleMesh, _distance: f64) -> TriangleMesh {
        mesh.clone()
    }

    pub fn apply_op(mesh_a: &TriangleMesh, mesh_b: &TriangleMesh, op: &CsgOpType) -> TriangleMesh {
        let csg_a = triangle_mesh_to_csg(mesh_a);
        let csg_b = triangle_mesh_to_csg(mesh_b);

        if csg_a.polygons.is_empty() || csg_b.polygons.is_empty() {
            return match op {
                CsgOpType::Union | CsgOpType::Subtract => mesh_a.clone(),
                CsgOpType::Intersect => empty_mesh(),
            };
        }

        let result = match op {
            CsgOpType::Union     => csg_a.union(&csg_b),
            CsgOpType::Subtract  => csg_a.difference(&csg_b),
            CsgOpType::Intersect => csg_a.intersection(&csg_b),
        };

        csg_to_triangle_mesh(result)
    }
}

// ── CsgKernel public API (dispatches to the active backend) ──────────────────

impl CsgKernel {
    pub fn new() -> Self { Self }

    pub fn apply_operation(
        &self,
        mesh_a: &TriangleMesh,
        mesh_b: &TriangleMesh,
        operation: &CsgOperation,
    ) -> TriangleMesh {
        #[cfg(feature = "manifold")]
        {
            manifold_backend::apply_op(mesh_a, mesh_b, &operation.op_type)
        }
        #[cfg(not(feature = "manifold"))]
        {
            csgrs_backend::apply_op(mesh_a, mesh_b, &operation.op_type)
        }
    }

    pub fn union(&self, mesh_a: &TriangleMesh, mesh_b: &TriangleMesh) -> TriangleMesh {
        self.apply_operation(mesh_a, mesh_b, &CsgOperation {
            op_type: CsgOpType::Union,
            tool_id: "internal".to_string(),
            tool_transform: None,
        })
    }

    pub fn subtract(&self, mesh_a: &TriangleMesh, mesh_b: &TriangleMesh) -> TriangleMesh {
        self.apply_operation(mesh_a, mesh_b, &CsgOperation {
            op_type: CsgOpType::Subtract,
            tool_id: "internal".to_string(),
            tool_transform: None,
        })
    }

    pub fn intersect(&self, mesh_a: &TriangleMesh, mesh_b: &TriangleMesh) -> TriangleMesh {
        self.apply_operation(mesh_a, mesh_b, &CsgOperation {
            op_type: CsgOpType::Intersect,
            tool_id: "internal".to_string(),
            tool_transform: None,
        })
    }

    pub fn apply_fillet(&self, mesh: &TriangleMesh, radius: f64) -> TriangleMesh {
        #[cfg(feature = "manifold")]
        { manifold_backend::apply_fillet(mesh, radius) }
        #[cfg(not(feature = "manifold"))]
        { csgrs_backend::apply_fillet(mesh, radius) }
    }

    pub fn apply_chamfer(&self, mesh: &TriangleMesh, distance: f64) -> TriangleMesh {
        #[cfg(feature = "manifold")]
        { manifold_backend::apply_chamfer(mesh, distance) }
        #[cfg(not(feature = "manifold"))]
        { csgrs_backend::apply_chamfer(mesh, distance) }
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
    fn default() -> Self { Self::new() }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::geometry::TriangleMesh;

    fn make_box_at(cx: f64, cy: f64, cz: f64, w: f64, h: f64, d: f64) -> TriangleMesh {
        let (hw, hh, hd) = (w / 2.0, h / 2.0, d / 2.0);
        TriangleMesh {
            vertices: vec![
                [cx-hw, cy-hh, cz-hd], [cx+hw, cy-hh, cz-hd],
                [cx+hw, cy+hh, cz-hd], [cx-hw, cy+hh, cz-hd],
                [cx-hw, cy-hh, cz+hd], [cx+hw, cy-hh, cz+hd],
                [cx+hw, cy+hh, cz+hd], [cx-hw, cy+hh, cz+hd],
            ],
            normals:   vec![],
            uvs:       vec![],
            triangles: vec![
                [0,2,1],[0,3,2],[1,6,5],[1,2,6],
                [5,7,4],[5,6,7],[4,3,0],[4,7,3],
                [3,6,2],[3,7,6],[4,1,5],[4,0,1],
            ],
        }
    }

    #[test]
    fn test_union_non_overlapping() {
        let k = CsgKernel::new();
        let a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = make_box_at(5.0, 5.0, 5.0, 1.0, 1.0, 1.0);
        assert!(!k.union(&a, &b).triangles.is_empty());
    }

    #[test]
    fn test_subtract_non_overlapping() {
        let k = CsgKernel::new();
        let a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = make_box_at(10.0,10.0,10.0, 1.0, 1.0, 1.0);
        assert!(!k.subtract(&a, &b).triangles.is_empty());
    }

    #[test]
    fn test_intersect_non_overlapping() {
        let k = CsgKernel::new();
        let a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let b = make_box_at(10.0,10.0,10.0, 1.0, 1.0, 1.0);
        assert!(k.intersect(&a, &b).triangles.is_empty());
    }

    #[test]
    fn test_union_partial_overlap() {
        let k = CsgKernel::new();
        let a = make_box_at(0.0, 0.0, 0.0, 4.0, 4.0, 4.0);
        let b = make_box_at(2.0, 2.0, 2.0, 4.0, 4.0, 4.0);
        assert!(!k.union(&a, &b).triangles.is_empty());
    }

    #[test]
    fn test_subtract_fully_inside() {
        let k = CsgKernel::new();
        let a = make_box_at(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
        let b = make_box_at(0.0, 0.0, 0.0,  2.0,  2.0,  2.0);
        assert!(!k.subtract(&a, &b).triangles.is_empty());
    }
}
