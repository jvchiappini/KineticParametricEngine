use std::collections::HashMap;
use glam::{DMat4, DVec4};
use kpe_schema::geometry::{GeometryNode, GeometryNodeType, TriangleMesh, SketchDef};
use kpe_schema::joint::Joint;
use crate::extrude::{extrude_sketch, revolve_sketch, sweep_sketch};
use crate::joint::JointEngine;

use super::primitives::{build_box, build_cylinder, build_sphere, empty_mesh};
use super::transform::local_matrix;

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

    pub fn build_with_transform(&self, node: &GeometryNode, parent_to_world: DMat4) -> TriangleMesh {
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
            GeometryNodeType::Compound | GeometryNodeType::Assembly(_) | GeometryNodeType::JointGroup => {
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

    fn build_jointed_children(&self, child: &GeometryNode, parent_world: DMat4) -> TriangleMesh {
        let joint = self.joints.iter().find(|j| j.child_id == child.id);
        if let Some(j) = joint {
            let engine = JointEngine::new();
            let jm = engine.compute_joint_matrix(j);
            let joint_world = parent_world * jm;
            let child_local = local_matrix(&child.transform);
            let child_world = joint_world * child_local;
            self.build_with_transform_skip_local(child, joint_world, child_world)
        } else {
            let child_local = local_matrix(&child.transform);
            let _child_world = parent_world * child_local;
            self.build_with_transform(child, parent_world)
        }
    }

    fn build_with_transform_skip_local(&self, node: &GeometryNode, _parent_to_world: DMat4, world: DMat4) -> TriangleMesh {
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
            GeometryNodeType::Compound | GeometryNodeType::Assembly(_) | GeometryNodeType::JointGroup => {
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

pub fn collect_sketches(node: &GeometryNode, map: &mut HashMap<String, SketchDef>) {
    if let GeometryNodeType::Sketch(s) = &node.node_type {
        map.insert(node.id.clone(), s.clone());
    }
    for child in &node.children {
        collect_sketches(child, map);
    }
}

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
