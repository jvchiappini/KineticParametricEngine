use kpe_schema::geometry::{
    BoxDef, CylinderDef, SphereDef, GeometryNode, GeometryNodeType, TriangleMesh, SketchDef, ExtrudeDef,
};
use crate::extrude::extrude_sketch;

pub struct MeshBuilder;

impl MeshBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build_from_node(&self, node: &GeometryNode) -> TriangleMesh {
        match &node.node_type {
            GeometryNodeType::Box(box_def) => self.build_box(box_def),
            GeometryNodeType::Cylinder(cyl_def) => self.build_cylinder(cyl_def),
            GeometryNodeType::Sphere(sphere_def) => self.build_sphere(sphere_def),
            GeometryNodeType::Mesh(mesh_def) => TriangleMesh {
                vertices: mesh_def.vertices.clone(),
                normals: vec![],
                uvs: vec![],
                triangles: mesh_def.indices.clone(),
            },
            GeometryNodeType::Sketch(sketch_def) => {
                let contours = crate::sketch::tessellate_sketch(sketch_def);
                let mut verts = Vec::new();
                for c in &contours {
                    for p in c {
                        verts.push([p.x, p.y, 0.0]);
                    }
                }
                TriangleMesh { vertices: verts, normals: vec![], uvs: vec![], triangles: vec![] }
            }
            GeometryNodeType::Extrude(extrude_def) => {
                let sketch = match find_sketch(node, extrude_def) {
                    Some(s) => s,
                    None => return TriangleMesh { vertices: vec![], normals: vec![], uvs: vec![], triangles: vec![] },
                };
                extrude_sketch(sketch, extrude_def)
            }
            GeometryNodeType::Compound => {
                let mut verts = Vec::new();
                let mut tris = Vec::new();
                for child in &node.children {
                    let child_mesh = self.build_from_node(child);
                    let base = verts.len() as u32;
                    verts.extend(child_mesh.vertices);
                    for t in child_mesh.triangles {
                        tris.push([t[0] + base, t[1] + base, t[2] + base]);
                    }
                }
                TriangleMesh {
                    vertices: verts,
                    normals: vec![],
                    uvs: vec![],
                    triangles: tris,
                }
            }
        }
    }

    pub fn build_box(&self, def: &BoxDef) -> TriangleMesh {
        let hw = def.width / 2.0;
        let hh = def.height / 2.0;
        let hd = def.depth / 2.0;

        let vertices = vec![
            [-hw, -hh, -hd], [ hw, -hh, -hd], [ hw,  hh, -hd], [-hw,  hh, -hd],
            [-hw, -hh,  hd], [ hw, -hh,  hd], [ hw,  hh,  hd], [-hw,  hh,  hd],
        ];

        let triangles = vec![
            [0, 2, 1], [0, 3, 2], [1, 6, 5], [1, 2, 6],
            [5, 7, 4], [5, 6, 7], [4, 3, 0], [4, 7, 3],
            [3, 6, 2], [3, 7, 6], [4, 1, 5], [4, 0, 1],
        ];

        TriangleMesh {
            vertices,
            normals: vec![],
            uvs: vec![],
            triangles,
        }
    }

    pub fn build_cylinder(&self, def: &CylinderDef) -> TriangleMesh {
        let segments = 64;
        let mut vertices = vec![[0.0, -def.height / 2.0, 0.0], [0.0, def.height / 2.0, 0.0]];
        let mut triangles = Vec::new();

        for i in 0..segments {
            let angle = (i as f64 / segments as f64) * std::f64::consts::TAU;
            let x = def.radius * angle.cos();
            let z = def.radius * angle.sin();

            vertices.push([x, -def.height / 2.0, z]);
            vertices.push([x, def.height / 2.0, z]);
        }

        for i in 0..segments {
            let next = (i + 1) % segments;
            let b0 = 2 + i * 2;
            let b1 = 2 + i * 2 + 1;
            let n0 = 2 + next * 2;
            let n1 = 2 + next * 2 + 1;

            triangles.push([b0, n0, n1]);
            triangles.push([b0, n1, b1]);
            triangles.push([0, n0, b0]);
            triangles.push([1, b1, n1]);
        }

        TriangleMesh {
            vertices,
            normals: vec![],
            uvs: vec![],
            triangles,
        }
    }

    pub fn build_sphere(&self, def: &SphereDef) -> TriangleMesh {
        let rings = 32;
        let segments = 64;
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();

        vertices.push([0.0, def.radius, 0.0]);

        for ring in 1..rings {
            let phi = (ring as f64 / rings as f64) * std::f64::consts::PI;
            for seg in 0..segments {
                let theta = (seg as f64 / segments as f64) * std::f64::consts::TAU;
                let x = def.radius * phi.sin() * theta.cos();
                let y = def.radius * phi.cos();
                let z = def.radius * phi.sin() * theta.sin();
                vertices.push([x, y, z]);
            }
        }

        vertices.push([0.0, -def.radius, 0.0]);

        for i in 0..segments {
            let next = (i + 1) % segments;
            triangles.push([0, 1 + i, 1 + next]);
        }

        for ring in 0..rings - 2 {
            for seg in 0..segments {
                let next = (seg + 1) % segments;
                let a0 = 1 + ring * segments + seg;
                let a1 = 1 + ring * segments + next;
                let b0 = 1 + (ring + 1) * segments + seg;
                let b1 = 1 + (ring + 1) * segments + next;

                triangles.push([a0 as u32, a1 as u32, b1 as u32]);
                triangles.push([a0 as u32, b1 as u32, b0 as u32]);
            }
        }

        let top = (vertices.len() - 1) as u32;
        let last_ring_start = 1 + (rings - 2) * segments;
        for i in 0..segments {
            let next = (i + 1) % segments;
            triangles.push([top, last_ring_start as u32 + next, last_ring_start as u32 + i]);
        }

        TriangleMesh {
            vertices,
            normals: vec![],
            uvs: vec![],
            triangles,
        }
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn find_sketch<'a>(node: &'a GeometryNode, extrude: &ExtrudeDef) -> Option<&'a SketchDef> {
    if let GeometryNodeType::Sketch(sketch) = &node.node_type {
        if node.id == extrude.sketch_id {
            return Some(sketch);
        }
    }
    for child in &node.children {
        if let Some(found) = find_sketch(child, extrude) {
            return Some(found);
        }
    }
    None
}
