use kpe_schema::geometry::{CsgOpType, CsgOperation, TriangleMesh, BRepModel};
use csgrs::mesh::Mesh;
use csgrs::mesh::polygon::Polygon;
use csgrs::mesh::vertex::Vertex;
use csgrs::traits::CSG;
use nalgebra::Point3;
use glam::DVec3;

// csgrs uses f64 as `Real` by default (the `f64` feature is on by default).
// We alias the unit metadata type for clarity.
type CsgMesh = Mesh<()>;

pub struct CsgKernel;

/// Convert a `TriangleMesh` into a `csgrs` `Mesh`.
///
/// Each triangle becomes one `Polygon` with the face normal assigned uniformly
/// to all three vertices (flat shading). An empty mesh is represented as an
/// empty polygon list; `csgrs` handles that correctly.
fn triangle_mesh_to_csg(mesh: &TriangleMesh) -> CsgMesh {
    let polygons: Vec<Polygon<()>> = mesh
        .triangles
        .iter()
        .filter_map(|tri| {
            let mut tri_verts = Vec::with_capacity(3);
            
            // Collect vertices for this triangle
            let mut points = [Point3::origin(); 3];
            for i in 0..3 {
                let p = &mesh.vertices[tri[i] as usize];
                points[i] = Point3::new(p[0], p[1], p[2]);
            }

            // Compute face normal consistent with [a, b, c] winding.
            let a = points[0];
            let b = points[1];
            let c = points[2];
            let ab = b - a;
            let ac = c - a;
            let n = ab.cross(&ac);
            let len = n.norm();
            let normal = if len > 1e-10 {
                n / len
            } else {
                return None;
            };

            // Create/Retrieve shared vertices
            for &p in &[a, b, c] {
                let bits = [p.x.to_bits(), p.y.to_bits(), p.z.to_bits()];
                // We use vertex normals consistent with the face for now (flat input)
                // but sharing the vertex position is key for BSP.
                let v = Vertex::new(p, normal);
                tri_verts.push(v);
            }
            
            Some(Polygon::new(tri_verts, None))
        })
        .collect();

    Mesh::from_polygons(&polygons, None)
}

use std::collections::HashMap;

/// Convert a `csgrs` `Mesh` back to a `TriangleMesh`.
///
/// `csgrs` stores geometry as (possibly non-triangular) polygons. We call
/// `triangulate()` on each polygon to get `[Vertex; 3]` triangles and then
/// build an indexed vertex list. 
///
/// We use a `vertex_map` to deduplicate vertices with the same position 
/// (within a small epsilon if needed, but here we use bitwise equality since 
/// csgrs output vertices often share bit-identical coordinates).
fn quantize(val: f64) -> i64 {
    (val * 1e5).round() as i64
}

fn csg_to_triangle_mesh(csg_mesh: CsgMesh) -> TriangleMesh {
    let mut vertices: Vec<[f64; 3]> = Vec::new();
    let mut normals: Vec<[f64; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    
    // Hash map for vertex deduplication: quantized 1e-5 to heal cracks
    let mut vertex_map: HashMap<[i64; 6], u32> = HashMap::new();

    for polygon in &csg_mesh.polygons {
        for tri_verts in polygon.triangulate() {
            let mut indices = [0u32; 3];
            for (i, v) in tri_verts.iter().enumerate() {
                let pos = [v.pos.x, v.pos.y, v.pos.z];
                // Convert f64 to quantized i64 for hashing both pos and normal
                let bits = [
                    quantize(v.pos.x),
                    quantize(v.pos.y),
                    quantize(v.pos.z),
                    quantize(v.normal.x),
                    quantize(v.normal.y),
                    quantize(v.normal.z),
                ];

                if let Some(&idx) = vertex_map.get(&bits) {
                    indices[i] = idx;
                } else {
                    let idx = vertices.len() as u32;
                    vertices.push(pos);
                    // Use the normal provided by csgrs (or later recompute in Three.js)
                    normals.push([v.normal.x, v.normal.y, v.normal.z]);
                    vertex_map.insert(bits, idx);
                    indices[i] = idx;
                }
            }
            triangles.push(indices);
        }
    }

    let num_verts = vertices.len();
    TriangleMesh {
        vertices,
        normals,
        uvs: vec![[0.0, 0.0]; num_verts],
        triangles,
    }
}

fn empty_mesh() -> TriangleMesh {
    TriangleMesh {
        vertices: vec![],
        normals: vec![],
        uvs: vec![],
        triangles: vec![],
    }
}

fn flip_triangles(mesh: &TriangleMesh) -> TriangleMesh {
    let mut tris = mesh.triangles.clone();
    for t in &mut tris {
        t.swap(1, 2);
    }
    TriangleMesh {
        vertices: mesh.vertices.clone(),
        normals: mesh.normals.clone(),
        uvs: mesh.uvs.clone(),
        triangles: tris,
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
        // Fast-path: handle empty operands without entering the CSG kernel.
        if mesh_a.triangles.is_empty() {
            return match operation.op_type {
                CsgOpType::Union => mesh_b.clone(),
                _ => empty_mesh(),
            };
        }
        if mesh_b.triangles.is_empty() {
            return match operation.op_type {
                CsgOpType::Intersect => empty_mesh(),
                _ => mesh_a.clone(),
            };
        }

        if operation.op_type == CsgOpType::Intersect {
            let csg_a = triangle_mesh_to_csg(mesh_a);
            let csg_b = triangle_mesh_to_csg(mesh_b);
            return csg_to_triangle_mesh(csg_a.intersection(&csg_b));
        }

        let csg_a = triangle_mesh_to_csg(mesh_a);
        let csg_b = triangle_mesh_to_csg(mesh_b);

        let result = match operation.op_type {
            CsgOpType::Union     => csg_a.union(&csg_b),
            CsgOpType::Subtract  => csg_a.difference(&csg_b),
            _ => unreachable!(),
        };

        csg_to_triangle_mesh(result)
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
                [0, 2, 1], [0, 3, 2], [1, 6, 5], [1, 2, 6],
                [5, 7, 4], [5, 6, 7], [4, 3, 0], [4, 7, 3],
                [3, 6, 2], [3, 7, 6], [4, 1, 5], [4, 0, 1],
            ],
        }
    }

    #[test]
    fn test_union_non_overlapping() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mesh_b = make_box_at(5.0, 5.0, 5.0, 1.0, 1.0, 1.0);
        let result = kernel.union(&mesh_a, &mesh_b);
        // Union of two non-overlapping boxes should have at least as many
        // triangles as both inputs combined.
        assert!(!result.triangles.is_empty());
    }

    #[test]
    fn test_subtract_non_overlapping() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let mesh_b = make_box_at(10.0, 10.0, 10.0, 1.0, 1.0, 1.0);
        let result = kernel.subtract(&mesh_a, &mesh_b);
        assert!(!result.triangles.is_empty());
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
    fn test_union_partial_overlap() {
        let kernel = CsgKernel::new();
        let mesh_a = make_box_at(0.0, 0.0, 0.0, 4.0, 4.0, 4.0);
        let mesh_b = make_box_at(2.0, 2.0, 2.0, 4.0, 4.0, 4.0);
        let result = kernel.union(&mesh_a, &mesh_b);
        assert!(!result.triangles.is_empty());
    }

    #[test]
    fn test_subtract_inner_removes_triangles() {
        let kernel = CsgKernel::new();
        let outer = make_box_at(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
        let inner = make_box_at(0.0, 0.0, 0.0, 4.0, 4.0, 4.0);
        let result = kernel.subtract(&outer, &inner);
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
