use std::collections::HashMap;
use glam::{DMat4, DVec3, DVec4};
use kpe_schema::geometry::{
    BoxDef, CylinderDef, SphereDef, GeometryNode, GeometryNodeType, TriangleMesh,
    SketchDef, TransformOp, FilletDef, ChamferDef,
};
use kpe_schema::joint::Joint;
use crate::extrude::{extrude_sketch, revolve_sketch, sweep_sketch};
use crate::joint::JointEngine;

pub struct MeshBuilder {
    sketches: HashMap<String, SketchDef>,
    joints: Vec<Joint>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self { sketches: HashMap::new(), joints: Vec::new() }
    }

    pub fn with_sketches(mut self, sketches: HashMap<String, SketchDef>) -> Self {
        self.sketches = sketches;
        self
    }

    pub fn with_joints(mut self, joints: Vec<Joint>) -> Self {
        self.joints = joints;
        self
    }

    pub fn build_from_node(&self, node: &GeometryNode) -> TriangleMesh {
        self.build_with_transform(node, DMat4::IDENTITY)
    }

    fn build_with_transform(&self, node: &GeometryNode, parent_to_world: DMat4) -> TriangleMesh {
        let local = local_matrix(&node.transform);
        let world = parent_to_world * local;

        let mut mesh = match &node.node_type {
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
                if let Some(extrude_def) = &sketch_def.extrude {
                    extrude_sketch(sketch_def, extrude_def)
                } else {
                    let contours = crate::sketch::tessellate_sketch(sketch_def);
                    let mut verts = Vec::new();
                    let mut tris = Vec::new();
                    for c in &contours {
                        if c.len() < 3 { continue; }
                        let base = verts.len() as u32;
                        for p in c {
                            verts.push([p.x, p.y, 0.0]);
                        }
                        let tri_indices = crate::sketch::triangulate_contour(c);
                        for t in &tri_indices {
                            tris.push([t[0] + base, t[1] + base, t[2] + base]);
                        }
                    }
                    TriangleMesh { vertices: verts, normals: vec![], uvs: vec![], triangles: tris }
                }
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
            GeometryNodeType::Fillet(fillet_def) => {
                let child_mesh = node.children.first()
                    .map(|c| self.build_jointed_children(c, world))
                    .unwrap_or_else(empty_mesh);
                let kernel = crate::csg::CsgKernel::new();
                kernel.apply_fillet(&child_mesh, fillet_def.radius)
            }
            GeometryNodeType::Chamfer(chamfer_def) => {
                let child_mesh = node.children.first()
                    .map(|c| self.build_jointed_children(c, world))
                    .unwrap_or_else(empty_mesh);
                let kernel = crate::csg::CsgKernel::new();
                kernel.apply_chamfer(&child_mesh, chamfer_def.distance)
            }
            GeometryNodeType::Compound | GeometryNodeType::Assembly(_) => {
                let mut verts = Vec::new();
                let mut tris = Vec::new();
                for child in &node.children {
                    let child_mesh = self.build_jointed_children(child, world);
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
        };

        // Apply world transform to all vertices
        if world != DMat4::IDENTITY {
            for v in &mut mesh.vertices {
                let p = world * DVec4::new(v[0], v[1], v[2], 1.0);
                v[0] = p.x / p.w;
                v[1] = p.y / p.w;
                v[2] = p.z / p.w;
            }
        }

        mesh
    }

    /// Build a child's mesh, applying any joint transform between this node and the child.
    fn build_jointed_children(&self, child: &GeometryNode, parent_world: DMat4) -> TriangleMesh {
        let joint = self.joints.iter().find(|j| j.child_id == child.id);
        if let Some(j) = joint {
            let engine = JointEngine::new();
            let jm = engine.compute_joint_matrix(j);
            let joint_world = parent_world * jm;
            // Pass the joint parent's world through; child's own transform is applied inside
            let child_local = local_matrix(&child.transform);
            let child_world = joint_world * child_local;
            self.build_with_transform_skip_local(child, joint_world, child_world)
        } else {
            let child_local = local_matrix(&child.transform);
            let child_world = parent_world * child_local;
            self.build_with_transform(child, parent_world)
        }
    }

    /// Like build_with_transform but uses the given world (skips recomputing from parent_to_world).
    fn build_with_transform_skip_local(&self, node: &GeometryNode, parent_to_world: DMat4, world: DMat4) -> TriangleMesh {
        // Same as build_with_transform, but avoids recomputing local*parent
        let mut mesh = match &node.node_type {
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
                if let Some(extrude_def) = &sketch_def.extrude {
                    extrude_sketch(sketch_def, extrude_def)
                } else {
                    let contours = crate::sketch::tessellate_sketch(sketch_def);
                    let mut verts = Vec::new();
                    let mut tris = Vec::new();
                    for c in &contours {
                        if c.len() < 3 { continue; }
                        let base = verts.len() as u32;
                        for p in c {
                            verts.push([p.x, p.y, 0.0]);
                        }
                        let tri_indices = crate::sketch::triangulate_contour(c);
                        for t in &tri_indices {
                            tris.push([t[0] + base, t[1] + base, t[2] + base]);
                        }
                    }
                    TriangleMesh { vertices: verts, normals: vec![], uvs: vec![], triangles: tris }
                }
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
            GeometryNodeType::Fillet(fillet_def) => {
                let child_mesh = node.children.first()
                    .map(|c| self.build_jointed_children(c, world))
                    .unwrap_or_else(empty_mesh);
                let kernel = crate::csg::CsgKernel::new();
                kernel.apply_fillet(&child_mesh, fillet_def.radius)
            }
            GeometryNodeType::Chamfer(chamfer_def) => {
                let child_mesh = node.children.first()
                    .map(|c| self.build_jointed_children(c, world))
                    .unwrap_or_else(empty_mesh);
                let kernel = crate::csg::CsgKernel::new();
                kernel.apply_chamfer(&child_mesh, chamfer_def.distance)
            }
            GeometryNodeType::Compound | GeometryNodeType::Assembly(_) => {
                let mut verts = Vec::new();
                let mut tris = Vec::new();
                for child in &node.children {
                    let child_mesh = self.build_jointed_children(child, world);
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
        };

        if world != DMat4::IDENTITY {
            for v in &mut mesh.vertices {
                let p = world * DVec4::new(v[0], v[1], v[2], 1.0);
                v[0] = p.x / p.w;
                v[1] = p.y / p.w;
                v[2] = p.z / p.w;
            }
        }

        mesh
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Compute local transform matrix ───────────────────────────────

fn local_matrix(tf: &Option<TransformOp>) -> DMat4 {
    match tf {
        Some(t) => {
            let mut mat = DMat4::IDENTITY;

            if let Some(trans) = &t.translation {
                mat = DMat4::from_translation(DVec3::new(trans[0], trans[1], trans[2]));
            }

            if let Some(rot) = &t.rotation {
                let rx = DMat4::from_rotation_x(rot[0].to_radians());
                let ry = DMat4::from_rotation_y(rot[1].to_radians());
                let rz = DMat4::from_rotation_z(rot[2].to_radians());
                mat = mat * rz * ry * rx;
            }

            if let Some(scale) = &t.scale {
                mat = mat * DMat4::from_scale(DVec3::new(scale[0], scale[1], scale[2]));
            }

            mat
        }
        None => DMat4::IDENTITY,
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
    let builder = MeshBuilder { sketches, joints: Vec::new() };
    builder.build_from_node(node)
}

pub fn build_mesh_from_node_with_joints(node: &GeometryNode, joints: &[Joint]) -> TriangleMesh {
    let mut sketches = HashMap::new();
    collect_sketches(node, &mut sketches);
    let builder = MeshBuilder { sketches, joints: joints.to_vec() };
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
