use std::collections::HashMap;
use kpe_schema::geometry::{
    BoxDef, CylinderDef, SphereDef, GeometryNode, GeometryNodeType, TriangleMesh,
    SketchDef,
};
use crate::extrude::{extrude_sketch, revolve_sketch, sweep_sketch};

pub struct MeshBuilder {
    sketches: HashMap<String, SketchDef>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self { sketches: HashMap::new() }
    }

    pub fn build_from_node(&self, node: &GeometryNode) -> TriangleMesh {
        match &node.node_type {
            GeometryNodeType::Box(box_def) => build_box(box_def),
            GeometryNodeType::Cylinder(cyl_def) => build_cylinder(cyl_def),
            GeometryNodeType::Sphere(sphere_def) => build_sphere(sphere_def),
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
                let sketch = match self.sketches.get(&extrude_def.sketch_id) {
                    Some(s) => s,
                    None => return empty_mesh(),
                };
                extrude_sketch(sketch, extrude_def)
            }
            GeometryNodeType::Revolve(revolve_def) => {
                let sketch = match self.sketches.get(&revolve_def.sketch_id) {
                    Some(s) => s,
                    None => return empty_mesh(),
                };
                revolve_sketch(sketch, revolve_def)
            }
            GeometryNodeType::Sweep(sweep_def) => {
                let sketch = match self.sketches.get(&sweep_def.sketch_id) {
                    Some(s) => s,
                    None => return empty_mesh(),
                };
                sweep_sketch(sketch, sweep_def)
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
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Collect sketches ─────────────────────────────────────────────

pub fn collect_sketches(node: &GeometryNode, map: &mut HashMap<String, SketchDef>) {
    if let GeometryNodeType::Sketch(s) = &node.node_type {
        map.insert(node.id.clone(), s.clone());
    }
    for child in &node.children {
        collect_sketches(child, map);
    }
}

// ── Free functions ──────────────────────────────────────────────

pub fn build_mesh_from_node(node: &GeometryNode) -> TriangleMesh {
    let mut sketches = HashMap::new();
    collect_sketches(node, &mut sketches);
    let builder = MeshBuilder { sketches };
    builder.build_from_node(node)
}

fn build_box(def: &BoxDef) -> TriangleMesh {
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

    TriangleMesh { vertices, normals: vec![], uvs: vec![], triangles }
}

fn build_cylinder(def: &CylinderDef) -> TriangleMesh {
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

    TriangleMesh { vertices, normals: vec![], uvs: vec![], triangles }
}

fn build_sphere(def: &SphereDef) -> TriangleMesh {
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

    TriangleMesh { vertices, normals: vec![], uvs: vec![], triangles }
}

fn empty_mesh() -> TriangleMesh {
    TriangleMesh { vertices: vec![], normals: vec![], uvs: vec![], triangles: vec![] }
}
